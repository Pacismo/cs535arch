using gui.Data;
using System.Net;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;


namespace gui.Controls
{
    public static class IntMath
    {
        public static int Pow(this int x, uint y)
        {
            return Enumerable.Repeat(x, (int) y).Aggregate(1, (a, e) => a * e);
        }
    }

    struct CacheViewRow
    {
        public string Address { get; set; }
        public uint Set { get; set; }
        public uint Way { get; set; }
        public bool? Dirty { get; set; }
        public string[] Data { get; set; }

        public CacheViewRow(uint address, uint set, uint way, bool dirty, int size, byte[] data)
        {
            Address = $"0x{address:X8}";
            Set = set;
            Way = way;
            Dirty = dirty;

            Data = new string[size];

            for (uint i = 0; i < size; ++i)
                Data[i] = $"{data[i]:X2}";
        }

        public CacheViewRow(uint set, uint way, int size)
        {
            Address = $"<INVALID>";
            Set = set;
            Way = way;
            Dirty = null;

            Data = new string[size];

            for (uint i = 0; i < size; ++i)
                Data[i] = $"";
        }
    }
    /// <summary>
    /// Interaction logic for CacheView.xaml
    /// </summary>
    public partial class CacheView : UserControl
    {
        Dictionary<string, Line?[]> caches = new();
        Dictionary<string, CacheConfig> configs;

        public CacheView()
        {
            InitializeComponent();
        }

        public void UpdateData(Cache[]? caches)
        {
            if (caches != null)
                foreach (var cache in caches)
                    this.caches[cache.name] = cache.lines;

            UpdateView();
        }

        public void SetConfiguration(Dictionary<string, CacheConfig> config)
        {
            configs = config;
            caches.Clear();
            Selector.Items.Clear();
            foreach (var pair in configs)
            {
                caches.Add(pair.Key, new Line?[2.Pow(pair.Value.set_bits) * pair.Value.ways]);
                Selector.Items.Add(pair.Key);
            }

            if (Selector.SelectedIndex < 0) Selector.SelectedIndex = 0;

            UpdateView();
        }

        void UpdateView()
        {
            Data.Items.Clear();

            IsEnabled = true;
            if (caches.Count == 0)
                IsEnabled = false;
            else
                ShowCache(Selector.SelectedItem as string ?? throw new NullReferenceException());
        }

        void ConfigureColumns(int line_width)
        {
            Data.Columns.Clear();
            Data.Columns.Add(new DataGridTextColumn() { Header = "Set", Binding = new Binding("Set") });
            Data.Columns.Add(new DataGridTextColumn() { Header = "Way", Binding = new Binding("Way") });
            Data.Columns.Add(new DataGridTextColumn() { Header = "Address", Binding = new Binding("Address") });
            Data.Columns.Add(new DataGridTextColumn() { Header = "Dirty", Binding = new Binding("Dirty") });

            for (int i = 0; i < line_width; ++i)
                Data.Columns.Add(new DataGridTextColumn() { Binding = new Binding($"Data[{i}]") });
        }

        void ShowCache(string cache_name)
        {
            Data.Items.Clear();

            Line?[] lines = caches[cache_name];
            CacheConfig config = configs[cache_name];

            if (lines.Length == 0) return;
            ConfigureColumns(2.Pow(config.offset_bits));

            for (uint i = 0; i < lines.Length; ++i)
            {
                Line? line = lines[i];
                if (line.HasValue)
                    Data.Items.Add(new CacheViewRow(line.Value.base_address, i / config.ways, i % config.ways, line.Value.dirty, 2.Pow(config.offset_bits), line.Value.data));
                else
                    Data.Items.Add(new CacheViewRow(i / config.ways, i % config.ways, 2.Pow(config.offset_bits)));
            }

        }

        private void Selector_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (Selector.SelectedItem is string selection)
                ShowCache(selection);
        }
    }
}

