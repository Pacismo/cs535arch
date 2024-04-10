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
    public struct Configuration
    {
        public uint miss_penalty;
        public uint volatile_penalty;
        public bool pipelining;
        public bool writethrough;

        public CacheConfig data_cache;
        public CacheConfig instruction_cache;

        public bool Validate()
        {
            return miss_penalty > 0 && volatile_penalty > 0 && data_cache.Validate() && instruction_cache.Validate();
        }

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

    struct PotentialUpdate<T>(T? result, int hash)
    {
        public T? result = result;
        public int hash = hash;
    }

    struct SimulationState()
    {
        public const string SEIS_SIM_BIN_PATH = "./bin/seis-sim";

        public Process? backend_process = null;
        public uint page_id = 0;
        public ViewUpdateFlags update = new();
        public Configuration running_config = new();

        public void Start(string binary_file, Configuration config)
        {
            running_config = config;
            File.WriteAllText("config.toml", Toml.FromModel(config.IntoToml()));
            string[] args = ["config.toml", binary_file, "-b"];
            backend_process = Process.Start(new ProcessStartInfo(SEIS_SIM_BIN_PATH, args)
            {
                RedirectStandardError = true,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                CreateNoWindow = true,
            });
        }

        public readonly bool IsRunning() => backend_process != null;

        public void Stop()
        {
            backend_process?.Kill();
            backend_process = null;
        }

        public void Command(string command)
        {
            if (backend_process == null)
                throw new InvalidOperationException("Backend process is not running");

            lock (backend_process)
                backend_process.StandardInput.WriteLine(command);
        }

        public string GetLine()
        {
            if (backend_process == null)
                throw new InvalidOperationException("Backend process is not running");

            string line;
            lock (backend_process)
                line = backend_process.StandardOutput.ReadLine() ?? throw new IOException("Pipe closed");

            return line;
        }

        public PotentialUpdate<T>? DeserializeResult<T>(string command, int? previous = null)
        {
            if (backend_process == null)
                throw new InvalidOperationException("Backend process is not running");

            string line;
            lock (backend_process)
            {
                backend_process.StandardInput.WriteLine(command);
                line = backend_process.StandardOutput.ReadLine() ?? throw new IOException("Pipe closed");
            }
            int hash = line.GetHashCode();
            if (previous == null || hash != previous)
                return new(JsonConvert.DeserializeObject<T>(line), hash);
            else
                return null;
        }
    }

    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
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

            if (!File.Exists("config.toml"))
                try
                {
                    string[] args = ["-e", "config.toml"];
                    Process proc = Process.Start(SimulationState.SEIS_SIM_BIN_PATH, args);
                    proc.WaitForExit();
                }
                catch { }
            LoadConfigurationFrom("config.toml");
        }

        private void Window_Closed(object sender, EventArgs e)
        {
            state.backend_process?.Kill();
            if (ValidateConfiguration())
                try
                {
                    File.WriteAllText("config.toml", Toml.FromModel(GetConfiguration().IntoToml()));
                }
                catch (Exception ex)
                {
                    new OkDialog("Failed To Store Configuration", $"There was an issue while saving this configuration\n{ex}").ShowDialog();
                }
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
                Application.Current.Dispatcher.Invoke(() => Overview.UpdateData(content.Value.result));
            }

            state.update.overview = false;
        }

        void UpdateRegistersView()
        {
            var registers = state.DeserializeResult<Registers>("regs", state.update.registers_hash);

            if (registers.HasValue)
            {
                state.update.registers_hash = registers.Value.hash;
                Application.Current.Dispatcher.Invoke(() => RegisterView_Table.UpdateData(registers.Value.result));
            }

            state.update.registers = false;
        }
        void UpdateCacheView()
        {
            var caches = state.DeserializeResult<Cache[]>("cache", state.update.cache_hash);

            if (caches.HasValue)
            {
                state.update.cache_hash = caches.Value.hash;
                Application.Current.Dispatcher.Invoke(() => CacheView_Grid.UpdateData(caches.Value.result));
            }
            state.update.cache = false;
        }
        void UpdateMemoryView()
        {
            var page = state.DeserializeResult<Data.Page>($"page {state.page_id}", state.update.memory_hash);

            if (page.HasValue)
            {
                state.update.memory_hash = page.Value.hash;
                Application.Current.Dispatcher.Invoke(() =>
                {
                    MemoryView_Grid.UpdateData(page.Value.result, state.page_id);
                    MemoryView_PageID.Text = state.page_id.ToString();
                });
            }

            state.update.memory = false;
        }
        void UpdateDisassemblyView()
        {
            var rows = state.DeserializeResult<DisassembledWord[]>($"disasm {state.page_id}", state.update.disassembly_hash);
            var pc = state.DeserializeResult<PcOnlyRegisterStruct>("regs")?.result.pc
                ?? throw new NullReferenceException();

            if (rows.HasValue)
            {
                state.update.disassembly_hash = rows.Value.hash;
                Application.Current.Dispatcher.Invoke(() =>
                {
                    DisassemblyView_Grid.UpdateData(state.page_id, rows.Value.result);
                    DisassemblyView_PageID.Text = state.page_id.ToString();
                });
            } else
                Application.Current.Dispatcher.Invoke(() => DisassemblyView_Grid.PC = pc);

            state.update.disassembly = false;
        }
        void UpdatePipelineView()
        {
            var pipe_stages =
                state.DeserializeResult<Dictionary<string, Dictionary<string, object>>>("pipe", state.update.pipeline_hash);

            if (pipe_stages.HasValue)
            {
                if (pipe_stages.Value.result == null)
                    throw new NullReferenceException();

                state.update.pipeline_hash = pipe_stages.Value.hash;

                Application.Current.Dispatcher.Invoke(() =>
                {
                    PipelineView.Fetch.UpdateData(pipe_stages.Value.result["fetch"]);
                    PipelineView.Decode.UpdateData(pipe_stages.Value.result["decode"]);
                    PipelineView.Execute.UpdateData(pipe_stages.Value.result["execute"]);
                    PipelineView.Memory.UpdateData(pipe_stages.Value.result["memory"]);
                    PipelineView.Writeback.UpdateData(pipe_stages.Value.result["writeback"]);
                });
            }

            state.update.pipeline = false;
        }

        void UpdateRootView()
        {
            if (!state.IsRunning())
                return;

            if (state.update.overview)
                ThreadPool.QueueUserWorkItem(_ => UpdateOverview());

            switch (Tabs.SelectedIndex)
            {
                case 1:
                    if (state.update.registers)
                        ThreadPool.QueueUserWorkItem(_ => UpdateRegistersView());
                    break;

                case 2:
                    if (state.update.cache)
                        ThreadPool.QueueUserWorkItem(_ => UpdateCacheView());
                    break;

                case 3:
                    if (state.update.memory)
                        ThreadPool.QueueUserWorkItem(_ => UpdateMemoryView());
                    break;

                case 4:
                    if (state.update.disassembly)
                        ThreadPool.QueueUserWorkItem(_ => UpdateDisassemblyView());
                    break;

                case 5:
                    if (state.update.pipeline)
                        ThreadPool.QueueUserWorkItem(_ => UpdatePipelineView());
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

            state.Command("run");
            // Await the ending of the simulation before refreshing
            state.GetLine();

            InvalidateView();
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
            if (state.page_id < ushort.MaxValue)
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
    }
}
