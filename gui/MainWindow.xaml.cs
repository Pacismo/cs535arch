using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Diagnostics;
using gui.Data;
using Microsoft.Win32;
using System.IO;
using gui.Controls;
using Tomlyn;
using Tomlyn.Model;
using Newtonsoft.Json;
using System.Windows.Input;

namespace gui
{
    /// <summary>
    /// The configuration to use when creating the simulation.
    /// 
    /// This is turned into a string and forwarded to the frontend when initializing the simulation.
    /// </summary>
    public struct Configuration
    {
        /// <summary>
        /// Penalty for a miss
        /// </summary>
        public uint miss_penalty;
        /// <summary>
        /// Penalty for a cache bypass
        /// </summary>
        public uint volatile_penalty;
        /// <summary>
        /// Whether to enable pipelining
        /// </summary>
        public bool pipelining;
        /// <summary>
        /// Whether to read memory to the cache on write
        /// </summary>
        public bool writethrough;

        /// <summary>
        /// The configuration to use for the data cache
        /// </summary>
        public CacheConfig data_cache;
        /// <summary>
        /// The configuration to use for the instruction cache
        /// </summary>
        public CacheConfig instruction_cache;

        /// <summary>
        /// Validate the configuration
        /// </summary>
        /// <returns>True if the configuration is valid</returns>
        public bool Validate()
        {
            return miss_penalty > 0 && volatile_penalty > 0 && data_cache.Validate() && instruction_cache.Validate();
        }

        /// <summary>
        /// Transforms the configuration into a TOML table
        /// </summary>
        /// <returns>A TOML table representing the configuration</returns>
        public TomlTable IntoToml()
        {
            TomlTable caches = new()
                    {
                        { "data", data_cache.IntoTomlTable() },
                        { "instruction", instruction_cache.IntoTomlTable() }
                    };

            return new()
                    {
                        { "miss_penalty", miss_penalty },
                        { "volatile_penalty", volatile_penalty },
                        { "pipelining", pipelining },
                        { "writethrough", writethrough },
                        { "cache", caches }
                    };
        }

        /// <summary>
        /// Transforms the TOML table into this object
        /// </summary>
        /// <param name="table">The table to read from</param>
        /// <returns>The Configuration object representing the table</returns>
        /// <exception cref="InvalidDataException">If the TOML data does not contain the required keys</exception>
        public static Configuration FromToml(TomlTable table)
        {
            TomlTable caches = table["cache"] as TomlTable
                ?? throw new InvalidDataException("Expected dictionary \"cache\"");

            return new()
            {
                data_cache = CacheConfig.FromToml(caches["data"] as TomlTable),
                instruction_cache = CacheConfig.FromToml(caches["instruction"] as TomlTable),

                miss_penalty = (uint) (table["miss_penalty"] as long?
                    ?? throw new InvalidDataException("Type mismatch for field \"miss_penalty\" (expected an integer)")),
                volatile_penalty = (uint) (table["volatile_penalty"] as long?
                    ?? throw new InvalidDataException("Type mismatch for field \"volatile_penalty_penalty\" (expected an integer)")),
                pipelining = (table["pipelining"] as bool?
                    ?? throw new InvalidDataException("Type mismatch for field \"pipelining\" (expected a boolean)")) == true,
                writethrough = (table["writethrough"] as bool?
                    ?? throw new InvalidDataException("Type mismatch for field \"writethrough\" (expected a boolean)")) == true
            };
        }
    }

    /// <summary>
    /// Represents a set of flags determining whether re-fetching data is necessary to update the controls.
    /// 
    /// Stores a boolean flag representing whether the control had previously been updated since the last clock.
    /// 
    /// Stores a hash to prevent re-writing the state to the control.
    /// </summary>
    struct ViewUpdateFlags()
    {
        public bool overview = true;
        public bool registers = true;
        public bool cache = true;
        public bool memory = true;
        public bool disassembly = true;
        public bool pipeline = true;

        public int? overview_hash = null;
        public int? registers_hash = null;
        public int? cache_hash = null;
        public int? memory_hash = null;
        public int? disassembly_hash = null;
        public int? pipeline_hash = null;

        /// <summary>
        /// Sets the flags to require an update.
        /// </summary>
        public void NeedUpdate()
        {
            overview = true;
            registers = true;
            cache = true;
            memory = true;
            disassembly = true;
            pipeline = true;
        }
    }

    /// <summary>
    /// Represents an update. Contains the result (if applicable) and the hash of the string used to generate the result.
    /// </summary>
    /// <typeparam name="T">The type of the result</typeparam>
    /// <param name="result">The result of an update</param>
    /// <param name="hash">The hash of the string used to generate the result</param>
    struct PotentialUpdate<T>(T result, int hash)
    {
        public T result = result;
        public int hash = hash;
    }

    /// <summary>
    /// Represents the state of the simulation backend.
    /// </summary>
    class SimulationState()
    {
        public const string SEIS_SIM_BIN_PATH = "bin/seis-sim";

        /// <summary>
        /// Mutex to prevent multiple reads
        /// </summary>
        public Mutex proc_mtx = new();
        /// <summary>
        /// The process being used to run the backend
        /// </summary>
        public Process? backend_process = null;
        Task? run_task;
        public uint page_id = 0;
        uint max_page = 65535;
        public ViewUpdateFlags update = new();
        public Configuration running_config = new();

        /// <summary>
        /// The number of pages in the memoryspace.
        /// </summary>
        public uint MaxPage { get { return max_page; } }

        /// <summary>
        /// Initializes the simulation
        /// </summary>
        /// <param name="binary_file">The binary file to run</param>
        /// <param name="config">The configuration to run the simulation</param>
        public void Start(string binary_file, Configuration config)
        {
            running_config = config;
            string[] args = ["run", "-i", Toml.FromModel(config.IntoToml()), binary_file, "-b"];
            backend_process = Process.Start(new ProcessStartInfo(SEIS_SIM_BIN_PATH, args)
            {
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                CreateNoWindow = true,
                ErrorDialog = true,
            });

            lock (proc_mtx)
            {
                Dictionary<string, uint> dict = DeserializeResult<Dictionary<string, uint>>("info pages")?.result!;
                max_page = dict["page_count"];
            }
        }

        /// <summary>
        /// Whether there is a running backend process
        /// </summary>
        /// <returns></returns>
        public bool IsRunning() => backend_process != null;

        /// <summary>
        /// Stops the runtime.
        /// </summary>
        public void Stop()
        {
            run_task = null;
            backend_process?.Kill();
            backend_process = null;
        }

        /// <summary>
        /// Submits a command to the backend
        /// </summary>
        /// <param name="command">The string command to send</param>
        /// <exception cref="InvalidOperationException">Thrown if the process is not running</exception>
        public void Command(string command)
        {
            if (backend_process == null)
                throw new InvalidOperationException("Backend process is not running");

            lock (proc_mtx)
                backend_process.StandardInput.WriteLine(command);
        }

        /// <summary>
        /// Reads a line from the stdout
        /// </summary>
        /// <returns>The line written by the simulation</returns>
        /// <exception cref="InvalidOperationException">Thrown if the process is not running</exception>
        /// <exception cref="IOException">If the read failed</exception>
        public string GetLine()
        {
            if (backend_process == null)
                throw new InvalidOperationException("Backend process is not running");

            string line;
            lock (proc_mtx)
                line = backend_process.StandardOutput.ReadLine() ?? throw new IOException("Pipe closed");

            return line;
        }

        /// <summary>
        /// Submits a command and reads a line. A hash is generated and the object is deserialized from the output.
        /// 
        /// Accepts a hash representing the previous time the object was requested. This is used to determine whether to deserialize the new object.
        /// 
        /// Not specifying the previous hash will always result in the object being deserialized.
        /// </summary>
        /// <typeparam name="T">The type to be deserialized</typeparam>
        /// <param name="command">The command to run</param>
        /// <param name="previous">The previous hash</param>
        /// <returns>A <see cref="PotentialUpdate{T}"/> representing the possibly deserialized result</returns>
        /// <exception cref="InvalidOperationException">If the process is not running</exception>
        /// <exception cref="IOException">If there was an error reading the output</exception>
        /// <exception cref="InvalidDataException">If the deserialization failed</exception>
        public PotentialUpdate<T>? DeserializeResult<T>(string command, int? previous = null)
        {
            if (backend_process == null)
                throw new InvalidOperationException("Backend process is not running");

            string line;
            lock (proc_mtx)
            {
                backend_process.StandardInput.WriteLine(command);
                line = backend_process.StandardOutput.ReadLine() ?? throw new IOException("Pipe closed");
            }
            int hash = line.GetHashCode();
            if (previous == null || hash != previous)
                return new(JsonConvert.DeserializeObject<T>(line) ?? throw new InvalidDataException("Error deserializing object"), hash);
            else
                return null;
        }

        /// <summary>
        /// Runs the simulation's code, returning when the backend says it's done.
        /// </summary>
        void run_proc()
        {
            lock (proc_mtx!)
            {
                backend_process!.StandardInput.WriteLine("run");
                backend_process.StandardOutput.ReadLine();
            }
            run_task = null;
        }

        /// <summary>
        /// Runs the simulation's code.
        /// </summary>
        /// <exception cref="InvalidOperationException">If the backend process is not running</exception>
        public void Run()
        {
            if (backend_process == null)
                throw new InvalidOperationException("Backend process is not running");
            if (run_task != null)
                return;

            var self = this;
            run_task = Task.Run(run_proc);
        }

        /// <summary>
        /// Stops running the code in the simulation
        /// </summary>
        public void Break()
        {
            if (run_task != null)
                backend_process!.StandardInput.WriteLine("stop");
            run_task = null;
        }
    }

    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        /// <summary>
        /// Allows for the use of a hotkey to clock the simulation.
        /// 
        /// By default, it is [SPACE].
        /// </summary>
        public static RoutedCommand ClockCommand = new();

        string binary_file = "";
        SimulationState state = new();

        private void ClockCommandHandler(object s, ExecutedRoutedEventArgs e)
        {
            if (!state.IsRunning())
                return;

            state.Command("clock 1");
            InvalidateView();
        }

        public MainWindow()
        {
            InitializeComponent();
            DataContext = this;
        }

        void SetBinary(string binary)
        {
            if (!File.Exists(binary))
                throw new FileNotFoundException($"File \"{binary}\" does not exist in the filesystem");

            binary_file = binary;
            Title = $"SEIS Simulation Frontend ({binary_file})";
        }

        private void Window_Loaded(object sender, RoutedEventArgs e)
        {
            foreach (TabItem tab in Tabs.Items)
                tab.Visibility = Visibility.Hidden;
            Tabs_Config.Visibility = Visibility.Visible;

            Tabs.SelectedIndex = 0;

            StopSim.IsEnabled = false;
            ResetSim.IsEnabled = false;
            Clock.IsEnabled = false;
            Run.IsEnabled = false;
            Overview.Visibility = Visibility.Hidden;

            var cli = Environment.GetCommandLineArgs();
            if (cli?.Length > 1)
                if (!File.Exists(cli[1]))
                    new OkDialog("Error Reading Configuration", $"Configuration file {cli[1]} does not exist").ShowDialog();
                else try
                    {
                        LoadConfigurationFrom(cli[1]);
                        return;
                    }
                    catch (Exception x)
                    {
                        new OkDialog("Error Reading Configuration", $"There was a problem loading the configuration from {cli[1]}\n{x}").ShowDialog();
                    }
        }

        private void Window_Closed(object sender, EventArgs e)
        {
            state.backend_process?.Kill();
        }

        private void Exit_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }

        private bool ValidateCacheMissPenalty()
        {
            try
            {
                uint value = uint.Parse(Penalty_CacheMiss.Text.Trim());
                if (value == 0)
                {
                    Penalty_CacheMiss_L.Foreground = new SolidColorBrush(Colors.Red);
                    return false;
                }
                else
                {
                    Penalty_CacheMiss_L.Foreground = new SolidColorBrush(Colors.Black);
                    return true;
                }
            }
            catch
            {
                Penalty_CacheMiss_L.Foreground = new SolidColorBrush(Colors.Red);
                return false;
            }
        }

        private bool ValidateVolatilePenalty()
        {
            try
            {
                uint value = uint.Parse(Penalty_Volatile.Text.Trim());
                if (value == 0)
                {
                    Penalty_Volatile_L.Foreground = new SolidColorBrush(Colors.Red);
                    return false;
                }
                else
                {
                    Penalty_Volatile_L.Foreground = new SolidColorBrush(Colors.Black);
                    return true;
                }
            }
            catch
            {
                Penalty_Volatile_L.Foreground = new SolidColorBrush(Colors.Red);
                return false;
            }
        }

        private void Penalty_CacheMiss_TextChanged(object sender, TextChangedEventArgs e)
        {
            ValidateCacheMissPenalty();
        }

        private void Penalty_Volatile_TextChanged(object sender, TextChangedEventArgs e)
        {
            ValidateVolatilePenalty();
        }

        private void OpenBinary_Click(object sender, RoutedEventArgs e)
        {
            OpenFileDialog openFileDialog = new()
            {
                CheckFileExists = true,
                Filter = "Binary File|*.bin",
                DefaultDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)
            };
            if (openFileDialog.ShowDialog() == true)
                SetBinary(openFileDialog.FileName);
        }

        private void OpenConfiguration_Click(object sender, RoutedEventArgs e)
        {
            OpenFileDialog openFileDialog = new()
            {
                CheckFileExists = true,
                Filter = "TOML File|*.toml",
                DefaultDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)
            };

            if (openFileDialog.ShowDialog() == true)
            {
                LoadConfigurationFrom(openFileDialog.FileName, true);
            }
        }

        private bool ValidateConfiguration()
        {
            return Conf_Cache_Data.IsValid() && Conf_Cache_Instruction.IsValid() && ValidateCacheMissPenalty() && ValidateVolatilePenalty();
        }

        bool LoadConfigurationFrom(string path, bool dialog = false)
        {
            try
            {
                TomlTable table = Toml.ToModel(File.ReadAllText(path));

                if (table.ContainsKey("simulation"))
                {
                    TomlTable simulation = table["simulation"] as TomlTable
                        ?? throw new InvalidDataException("Key \"simulation\" must be a table");

                    if (simulation.ContainsKey("binary"))
                    {
                        string binary = simulation["binary"] as string
                            ?? throw new InvalidDataException("Key \"simulation.binary\" must be a string");

                        if (!Path.IsPathFullyQualified(binary))
                            binary = Path.Join(Path.GetDirectoryName(path), binary);

                        SetBinary(binary);
                    }
                }

                SetConfiguration(Configuration.FromToml(table));
                return true;
            }
            catch (IOException ex)
            {
                if (dialog)
                    new OkDialog("Error Opening File", $"There was an issue while opening the file, \"{path}\"\n{ex}").ShowDialog();
                return false;
            }
            catch (Exception ex)
            {
                if (dialog)
                    new OkDialog("Error Parsing File", $"There was an issue while loading the configuration from \"{path}\"\n{ex}").ShowDialog();
                return false;
            }
        }

        void SetConfiguration(Configuration conf)
        {
            if (!conf.Validate())
                throw new InvalidDataException("The configuration provided is not valid");

            Penalty_CacheMiss.Text = conf.miss_penalty.ToString();
            Penalty_Volatile.Text = conf.volatile_penalty.ToString();

            Pipelining_Enable.IsChecked = conf.pipelining;
            Writethrough_Enable.IsChecked = conf.writethrough;

            Conf_Cache_Data.SetConfiguration(conf.data_cache);
            Conf_Cache_Instruction.SetConfiguration(conf.instruction_cache);
        }

        Configuration GetConfiguration()
        {
            if (!ValidateConfiguration())
                throw new InvalidOperationException("The current configuration is not valid");

            return new()
            {
                writethrough = Writethrough_Enable.IsChecked == true,
                pipelining = Pipelining_Enable.IsChecked == true,
                miss_penalty = uint.Parse(Penalty_CacheMiss.Text),
                volatile_penalty = uint.Parse(Penalty_Volatile.Text),
                data_cache = Conf_Cache_Data.GetConfiguration(),
                instruction_cache = Conf_Cache_Instruction.GetConfiguration()
            };
        }

        private void SaveConfiguration_Click(object sender, RoutedEventArgs e)
        {
            if (!ValidateConfiguration())
            {
                new OkDialog("Invalid Configuration", "The configuration provided is not valid and cannot be saved.").ShowDialog();
                return;
            }

            SaveFileDialog saveFileDialog = new()
            {
                Filter = "TOML File|*.toml",
                DefaultDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)
            };

            if (saveFileDialog.ShowDialog() == true)
            {
                try
                {
                    File.WriteAllText(saveFileDialog.FileName, Toml.FromModel(GetConfiguration().IntoToml()));
                }
                catch (Exception ex)
                {
                    new OkDialog("Error Saving File", $"There was an exception while trying to save the configuration to \"{saveFileDialog.FileName}\"\n{ex}").ShowDialog();
                }
            }
        }

        private void StartSim_Click(object sender, RoutedEventArgs e)
        {
            if (!ValidateConfiguration())
            {
                new OkDialog("Failed to Launch", "The current configuration is not valid; failed to launch process.").ShowDialog();
                return;
            }
            if (binary_file == null || binary_file.Length == 0 || !File.Exists(binary_file))
            {
                new OkDialog("Failed to Launch", "Please select a binary image file.").ShowDialog();
                return;
            }

            try
            {
                state = new();
                state.Start(binary_file, GetConfiguration());
            }
            catch (Exception ex)
            {
                new OkDialog("Failed to Launch", $"There was a problem while launching the backend.\n{ex}").ShowDialog();
                return;
            }

            foreach (TabItem tab in Tabs.Items) tab.Visibility = Visibility.Visible;
            Tabs_Config_Content.IsEnabled = false;
            Tabs.SelectedIndex = 1;

            StartSim.IsEnabled = false;
            StopSim.IsEnabled = true;
            ResetSim.IsEnabled = true;
            Clock.IsEnabled = true;
            Run.IsEnabled = true;
            Overview.Visibility = Visibility.Visible;

            OpenBinary.IsEnabled = false;
            OpenConfiguration.IsEnabled = false;

            CacheView_Grid.SetConfiguration(new()
            {
                { "data", state.running_config.data_cache },
                { "instruction", state.running_config.instruction_cache }
            });
        }

        private void StopSim_Click(object sender, RoutedEventArgs e)
        {
            foreach (TabItem tab in Tabs.Items) tab.Visibility = Visibility.Hidden;
            Tabs_Config.Visibility = Visibility.Visible;
            Tabs_Config_Content.IsEnabled = true;
            Tabs.SelectedIndex = 0;

            StartSim.IsEnabled = true;
            StopSim.IsEnabled = false;
            ResetSim.IsEnabled = false;
            Clock.IsEnabled = false;
            Run.IsEnabled = false;
            Overview.Visibility = Visibility.Hidden;

            OpenBinary.IsEnabled = true;
            OpenConfiguration.IsEnabled = true;

            state.Stop();
        }

        private void ResetSim_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                state.Stop();
                state = new();
                state.Start(binary_file, GetConfiguration());
            }
            catch (Exception ex)
            {
                new OkDialog("Failed to Reset Process", $"There was an issue while restarting the simulation.\n{ex}").ShowDialog();
            }

            InvalidateView();
        }

        void UpdateOverview()
        {
            var content = state.DeserializeResult<OverviewContent>("stats", state.update.overview_hash);

            if (content.HasValue)
            {
                state.update.overview_hash = content.Value.hash;
                Application.Current.Dispatcher.InvokeAsync(() => Overview.UpdateData(content.Value.result));
            }

            state.update.overview = false;
        }

        void UpdateRegistersView()
        {
            state.update.registers = false;

            var registers = state.DeserializeResult<Registers>("regs", state.update.registers_hash);

            if (registers.HasValue)
            {
                state.update.registers_hash = registers.Value.hash;
                Application.Current.Dispatcher.InvokeAsync(() => RegisterView_Table.UpdateData(registers.Value.result));
            }
        }
        void UpdateCacheView()
        {
            state.update.cache = false;

            var caches = state.DeserializeResult<Cache[]>("cache", state.update.cache_hash);

            if (caches.HasValue)
            {
                state.update.cache_hash = caches.Value.hash;
                Application.Current.Dispatcher.InvokeAsync(() => CacheView_Grid.UpdateData(caches.Value.result));
            }
        }
        void UpdateMemoryView()
        {
            state.update.memory = false;

            var page = state.DeserializeResult<Data.Page>($"page {state.page_id}", state.update.memory_hash);

            if (page.HasValue)
            {
                state.update.memory_hash = page.Value.hash;
                Application.Current.Dispatcher.InvokeAsync(() =>
                {
                    MemoryView_Grid.UpdateData(page.Value.result, state.page_id);
                    MemoryView_PageID.Content = state.page_id.ToString();
                });
            }
        }
        void UpdateDisassemblyView()
        {
            state.update.disassembly = false;

            var rows = state.DeserializeResult<DisassembledWord[]>($"disasm {state.page_id}", state.update.disassembly_hash);
            var pc = state.DeserializeResult<PcOnlyRegisterStruct>("regs")?.result.pc
                ?? throw new NullReferenceException();

            if (rows.HasValue)
            {
                state.update.disassembly_hash = rows.Value.hash;
                Application.Current.Dispatcher.InvokeAsync(() =>
                {
                    DisassemblyView_Grid.UpdateData(state.page_id, rows.Value.result, pc);
                    DisassemblyView_PageID.Content = state.page_id.ToString();
                });
            }
            else
                Application.Current.Dispatcher.Invoke(() => DisassemblyView_Grid.PC = pc);
        }
        void UpdatePipelineView()
        {
            state.update.pipeline = false;

            var pipe_stages =
                state.DeserializeResult<Dictionary<string, Dictionary<string, object>>>("pipe", state.update.pipeline_hash);

            if (pipe_stages.HasValue)
            {
                if (pipe_stages.Value.result == null)
                    throw new NullReferenceException();

                state.update.pipeline_hash = pipe_stages.Value.hash;

                Application.Current.Dispatcher.InvokeAsync(() =>
                {
                    PipelineView.Fetch.UpdateData(pipe_stages.Value.result["fetch"]);
                    PipelineView.Decode.UpdateData(pipe_stages.Value.result["decode"]);
                    PipelineView.Execute.UpdateData(pipe_stages.Value.result["execute"]);
                    PipelineView.Memory.UpdateData(pipe_stages.Value.result["memory"]);
                    PipelineView.Writeback.UpdateData(pipe_stages.Value.result["writeback"]);
                });
            }
        }

        void UpdateRootView()
        {
            if (!state.IsRunning())
                return;

            if (state.update.overview)
                Task.Run(UpdateOverview);

            switch (Tabs.SelectedIndex)
            {
                case 1:
                    if (state.update.registers)
                        Task.Run(UpdateRegistersView);
                    break;

                case 2:
                    if (state.update.cache)
                        Task.Run(UpdateCacheView);
                    break;

                case 3:
                    if (state.update.memory)
                        Task.Run(UpdateMemoryView);
                    break;

                case 4:
                    if (state.update.disassembly)
                        Task.Run(UpdateDisassemblyView);
                    break;

                case 5:
                    if (state.update.pipeline)
                        Task.Run(UpdatePipelineView);
                    break;

                default:
                    break;
            }
        }

        void InvalidateView()
        {
            state.update.NeedUpdate();
            UpdateRootView();
        }

        private void Clock_Click(object sender, RoutedEventArgs e)
        {
            if (!state.IsRunning())
                throw new InvalidOperationException("Backend process is not running");

            state.Command("clock 1");

            InvalidateView();
        }

        private void Run_Click(object sender, RoutedEventArgs e)
        {
            if (!state.IsRunning())
                throw new InvalidOperationException("Backend process is not running");

            Task.Run(() =>
            {
                state.Run();
                Application.Current.Dispatcher.Invoke(InvalidateView);
            });
        }
        private void Break_Click(object sender, RoutedEventArgs e)
        {
            state.Break();
        }

        private void Tabs_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            UpdateRootView();
        }

        private void MemoryView_Previous_Click(object sender, RoutedEventArgs e)
        {
            if (!state.IsRunning())
                throw new InvalidOperationException("Backend process is not running");
            if (state.page_id > 0)
            {
                state.page_id -= 1;
                state.update.memory = true;
                state.update.memory_hash = null;
                state.update.disassembly = true;
                state.update.disassembly_hash = null;
                UpdateRootView();
            }
        }

        private void MemoryView_Next_Click(object sender, RoutedEventArgs e)
        {
            if (!state.IsRunning())
                throw new InvalidOperationException("Backend process is not running");
            if (state.page_id < state.MaxPage - 1)
            {
                state.page_id += 1;
                state.update.memory = true;
                state.update.memory_hash = null;
                state.update.disassembly = true;
                state.update.disassembly_hash = null;
                UpdateRootView();
            }
        }

        private void About_Click(object sender, RoutedEventArgs e)
        {
            new About().ShowDialog();
        }

        private void MemoryView_PageID_MouseDoubleClick(object sender, MouseButtonEventArgs e)
        {
            PageSelector selector = new(state.page_id, state.MaxPage);
            selector.ShowDialog();

            uint? pid = selector.GetPageID();
            if (pid.HasValue && pid.Value != state.page_id)
            {
                state.page_id = pid.Value;
                state.update.memory = true;
                state.update.memory_hash = null;
                state.update.disassembly = false;
                state.update.disassembly_hash = null;
                UpdateRootView();
            }
        }
    }
}
