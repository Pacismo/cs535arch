﻿using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;
using System.Runtime;
using System.Diagnostics;
using gui.Data;
using Microsoft.Win32;
using Microsoft.VisualBasic;
using System.IO;
using System.Windows.Controls.Primitives;
using gui.Controls;
using Tomlyn;
using Tomlyn.Model;
using System.Security.Cryptography.X509Certificates;

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
            TomlTable caches = table["cache"] as TomlTable ?? throw new InvalidDataException("Expected dictionary \"cache\"");

            return new()
            {
                data_cache = CacheConfig.FromToml(caches["data"] as TomlTable),
                instruction_cache = CacheConfig.FromToml(caches["instruction"] as TomlTable),

                miss_penalty = (uint)(table["miss_penalty"] as long? ?? throw new InvalidDataException("Type mismatch for field \"miss_penalty\" (expected an integer)")),
                volatile_penalty = (uint)(table["volatile_penalty"] as long? ?? throw new InvalidDataException("Type mismatch for field \"volatile_penalty_penalty\" (expected an integer)")),
                pipelining = (table["pipelining"] as bool? ?? throw new InvalidDataException("Type mismatch for field \"pipelining\" (expected a boolean)")) == true,
                writethrough = (table["writethrough"] as bool? ?? throw new InvalidDataException("Type mismatch for field \"writethrough\" (expected a boolean)")) == true
            };
        }
    }

    struct ViewUpdateFlags()
    {
        public bool overview = true;
        public bool registers = true;
        public bool cache = true;
        public bool memory = true;
        public bool pipeline = true;
    }

    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        const string SEIS_SIM_BIN_PATH = "seis-sim";

        string binary_file = "";
        Process? backend_process;
        bool binary = false;
        uint page_id;
        Data.Page loaded_page;
        ViewUpdateFlags update_flags;

        public MainWindow()
        {
            InitializeComponent();
        }

        private void Window_Loaded(object sender, RoutedEventArgs e)
        {
            foreach (TabItem tab in Tabs.Items) tab.IsEnabled = false;
            Tabs_Config.IsEnabled = true;
            Tabs.SelectedIndex = 0;

            StopSim.IsEnabled = false;
            ResetSim.IsEnabled = false;
            Clock.IsEnabled = false;
            Run.IsEnabled = false;

            if (!System.IO.File.Exists("config.toml"))
                try
                {
                    string[] args = ["-e", "config.toml"];
                    Process proc = Process.Start(SEIS_SIM_BIN_PATH, args);
                    proc.WaitForExit();
                }
                catch { }
            LoadConfigurationFrom("config.toml");
        }

        private void Window_Closed(object sender, EventArgs e)
        {
            backend_process?.Kill();
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

        private void MemoryView_BinaryView_Checked(object sender, RoutedEventArgs e)
        {
            binary = MemoryView_Binary.IsChecked.HasValue ? MemoryView_Binary.IsChecked.Value : false;
            MemoryView.UpdateData(loaded_page, page_id, binary);
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
            {
                binary_file = openFileDialog.FileName;
                Title = $"SEIS Simulation Frontend ({binary_file})";
            }
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
                SetConfiguration(Configuration.FromToml(Toml.ToModel(File.ReadAllText(path))));
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
                File.WriteAllText("config.toml", Toml.FromModel(GetConfiguration().IntoToml()));
                string[] args = ["config.toml", binary_file, "-b"];
                backend_process = Process.Start(new ProcessStartInfo(SEIS_SIM_BIN_PATH, args)
                {
                    RedirectStandardError = true,
                    RedirectStandardInput = true,
                    RedirectStandardOutput = true,
                    CreateNoWindow = true,
                });
            }
            catch (Exception ex)
            {
                new OkDialog("Failed to Launch", $"There was a problem while launching the backend.\n{ex}").ShowDialog();
                return;
            }

            foreach (TabItem tab in Tabs.Items) tab.IsEnabled = true;
            Tabs_Config_Content.IsEnabled = false;
            Tabs.SelectedIndex = 1;

            StartSim.IsEnabled = false;
            StopSim.IsEnabled = true;
            ResetSim.IsEnabled = true;
            Clock.IsEnabled = true;
            Run.IsEnabled = true;

            OpenBinary.IsEnabled = false;
            OpenConfiguration.IsEnabled = false;
        }

        private void StopSim_Click(object sender, RoutedEventArgs e)
        {
            foreach (TabItem tab in Tabs.Items) tab.IsEnabled = false;
            Tabs_Config.IsEnabled = true;
            Tabs_Config_Content.IsEnabled = true;
            Tabs.SelectedIndex = 0;

            StartSim.IsEnabled = true;
            StopSim.IsEnabled = false;
            ResetSim.IsEnabled = false;
            Clock.IsEnabled = false;
            Run.IsEnabled = false;

            OpenBinary.IsEnabled = true;
            OpenConfiguration.IsEnabled = true;

            backend_process?.Kill();
        }

        private void ResetSim_Click(object sender, RoutedEventArgs e)
        {
            backend_process?.Kill();
            try
            {
                string[] args = ["config.toml", binary_file, "-b"];
                backend_process = Process.Start(new ProcessStartInfo(SEIS_SIM_BIN_PATH, args)
                {
                    RedirectStandardError = true,
                    RedirectStandardInput = true,
                    RedirectStandardOutput = true,
                    CreateNoWindow = true,
                });
            }
            catch (Exception ex)
            {
                new OkDialog("Failed to Reset Process", $"There was an issue while restarting the simulation.\n{ex}").ShowDialog();
            }
        }

        void UpdateOverview()
        {
            // TODO: update view
            update_flags.overview = false;
        }
        void UpdateRegistersView()
        {
            // TODO: update view
            update_flags.registers = false;
        }
        void UpdateCacheView()
        {
            // TODO: update view
            update_flags.cache = false;
        }
        void UpdateMemoryView()
        {
            // TODO: update view
            update_flags.memory = false;
        }
        void UpdatePipelineView()
        {
            // TODO: update view
            update_flags.pipeline = false;
        }

        void UpdateAllViews()
        {
            UpdateOverview();
            UpdateRegistersView();
            UpdateCacheView();
            UpdateMemoryView();
            UpdatePipelineView();
        }

        private void Clock_Click(object sender, RoutedEventArgs e)
        {
            if (backend_process == null)
                throw new InvalidOperationException("Backend process has not started!");

            backend_process.StandardInput.WriteLine("clock 1");
        }

        private void Run_Click(object sender, RoutedEventArgs e)
        {
            if (backend_process == null)
                throw new InvalidOperationException("Backend process has not started!");

            // TODO: Figure out how to have this process await the "done" message

            backend_process.StandardInput.WriteLine("run");
            // Await the ending of the simulation before refreshing
            backend_process.StandardOutput.ReadLine();
        }
    }
}
