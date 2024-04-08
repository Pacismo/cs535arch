using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Tomlyn.Model;

namespace gui.Controls
{
    public struct CacheConfig(bool enabled = false, uint off = 0, uint set = 0, uint ways = 0)
    {
        public bool is_enabled = enabled;
        public uint offset_bits = off;
        public uint set_bits = set;
        public uint ways = ways;

        public TomlTable IntoTomlTable()
        {

            if (is_enabled)
            {
                return new()
                {
                    { "mode", "associative" },
                    { "set_bits", set_bits },
                    { "offset_bits", offset_bits },
                    { "ways", ways }
                };
            }
            else
            {
                return new() {
                    {"mode", "disabled"}
                };
            }
        }

        public static CacheConfig FromToml(TomlTable? table)
        {
            if (table != null)
            {
                string? mode = table["mode"] as string;

                if (mode == "associative")
                {
                    return new CacheConfig()
                    {
                        is_enabled = true,
                        offset_bits = (uint)(table["offset_bits"] as long? ?? throw new InvalidDataException("Expected integer field \"offset_bits\" where \"mode\" is \"associative\"")),
                        set_bits = (uint)(table["set_bits"] as long? ?? throw new InvalidDataException("Expected integer field \"set_bits\" where \"mode\" is \"associative\"")),
                        ways = (uint)(table["ways"] as long? ?? throw new InvalidDataException("Expected integer field \"ways\" where \"mode\" is \"associative\""))
                    };
                }
                else if (mode == "disabled")
                {
                    return new();
                }
                else
                    throw new InvalidDataException($"Expected field \"mode\" to equal \"associative\" or \"disabled\", not {mode}");
            }
            else
            {
                return new()
                {
                    is_enabled = false,
                };
            }
        }

        public bool Validate()
        {
            if (is_enabled)
                return offset_bits >= 2 && set_bits + offset_bits <= 32 && ways > 0 && ways <= 16 && (ways % 2 != 1 || ways == 1);
            else
                return true;
        }
    }

    /// <summary>
    /// Interaction logic for CacheConfigurator.xaml
    /// </summary>
    public partial class CacheConfigurator : UserControl
    {
        public string Title { get; set; }

        public CacheConfigurator()
        {
            Title = "Cache Configurator";
            InitializeComponent();
            DataContext = this;

            OffsetBits.IsEnabled = false;
            SetBits.IsEnabled = false;
            Ways.IsEnabled = false;
        }

        public void SetConfiguration(CacheConfig config)
        {
            if (!config.Validate())
                throw new InvalidDataException("The provided configuration is not valid");

            if (config.is_enabled)
            {
                Enable.IsChecked = true;

                SetBits.Text = config.set_bits.ToString();
                SetBits.IsEnabled = true;

                OffsetBits.Text = config.offset_bits.ToString();
                OffsetBits.IsEnabled = true;

                Ways.Text = config.ways.ToString();
                Ways.IsEnabled = true;
            }
            else
            {
                Enable.IsChecked = false;

                SetBits.Text = "";
                SetBits_L.Foreground = new SolidColorBrush(Colors.Black);
                SetBits.IsEnabled = false;

                OffsetBits.Text = "";
                OffsetBits.IsEnabled = false;
                OffsetBits_L.Foreground = new SolidColorBrush(Colors.Black);

                Ways.Text = "";
                Ways.IsEnabled = false;
                Ways_L.Foreground = new SolidColorBrush(Colors.Black);
            }
        }

        public CacheConfig GetConfiguration()
        {
            if (!IsValid())
            {
                throw new InvalidOperationException("Configuration is not valid for this object");
            }

            if (Enable.IsChecked == true)
            {
                return new CacheConfig
                {
                    is_enabled = true,
                    offset_bits = uint.Parse(OffsetBits.Text.Trim()),
                    set_bits = uint.Parse(SetBits.Text.Trim()),
                    ways = uint.Parse(Ways.Text.Trim()),
                };
            }
            else
            {
                return new CacheConfig
                {
                    is_enabled = false,
                    offset_bits = 0,
                    set_bits = 0,
                    ways = 0,
                };
            }
        }

        public bool IsValid()
        {
            bool is_valid = true;

            if (Enable.IsChecked != true)
            {
                return true;
            }

            try
            {
                uint value = uint.Parse(OffsetBits.Text.Trim());
                if (value > 32 || value < 2)
                {
                    is_valid = false;
                    OffsetBits_L.Foreground = new SolidColorBrush(Colors.Red);
                }
                else
                    OffsetBits_L.Foreground = new SolidColorBrush(Colors.Black);
            }
            catch
            {
                is_valid = false;
                OffsetBits_L.Foreground = new SolidColorBrush(Colors.Red);
            }
            try
            {
                if (uint.Parse(SetBits.Text.Trim()) > 30)
                {
                    is_valid = false;
                    SetBits_L.Foreground = new SolidColorBrush(Colors.Red);
                }
                else
                    SetBits_L.Foreground = new SolidColorBrush(Colors.Black);
            }
            catch
            {
                is_valid = false;
                SetBits_L.Foreground = new SolidColorBrush(Colors.Red);
            }
            try
            {
                uint value = uint.Parse(Ways.Text.Trim());
                if (value >= 1 && value <= 16 && (value % 2 == 0 || value == 1))
                    Ways_L.Foreground = new SolidColorBrush(Colors.Black);
                else
                {
                    is_valid = false;
                    Ways_L.Foreground = new SolidColorBrush(Colors.Red);
                }
            }
            catch
            {
                is_valid = false;
                Ways_L.Foreground = new SolidColorBrush(Colors.Red);
            }

            return is_valid;
        }

        private void Enable_Click(object sender, RoutedEventArgs e)
        {
            if (Enable.IsChecked == true)
            {
                OffsetBits.IsEnabled = true;
                SetBits.IsEnabled = true;
                Ways.IsEnabled = true;
            }
            else
            {
                OffsetBits.IsEnabled = false;
                SetBits.IsEnabled = false;
                Ways.IsEnabled = false;
            }
        }

        private void OffsetBits_TextChanged(object sender, TextChangedEventArgs e)
        {
            try
            {
                uint value = uint.Parse(OffsetBits.Text.Trim());
                if (value > 32 || value < 2)
                    OffsetBits_L.Foreground = new SolidColorBrush(Colors.Red);
                else
                    OffsetBits_L.Foreground = new SolidColorBrush(Colors.Black);
            }
            catch
            {
                OffsetBits_L.Foreground = new SolidColorBrush(Colors.Red);
            }
        }

        private void SetBits_TextChanged(object sender, TextChangedEventArgs e)
        {
            try
            {
                if (uint.Parse(SetBits.Text.Trim()) > 30)
                    SetBits_L.Foreground = new SolidColorBrush(Colors.Red);
                else
                    SetBits_L.Foreground = new SolidColorBrush(Colors.Black);
            }
            catch
            {
                SetBits_L.Foreground = new SolidColorBrush(Colors.Red);
            }
        }

        private void Ways_TextChanged(object sender, TextChangedEventArgs e)
        {
            try
            {
                uint value = uint.Parse(Ways.Text.Trim());
                if (value >= 1 && value <= 16 && (value % 2 == 0 || value == 1))
                    Ways_L.Foreground = new SolidColorBrush(Colors.Black);
                else
                    Ways_L.Foreground = new SolidColorBrush(Colors.Red);
            }
            catch
            {
                Ways_L.Foreground = new SolidColorBrush(Colors.Red);
            }
        }
    }
}
