using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;

namespace gui.Controls
{
    public struct MemoryViewRow
    {
        public string Address { get; set; }
        public string[] Byte { get; set; }

        public MemoryViewRow(uint address, byte[] bytes)
        {
            Address = $"0x{address:X8}";
            Byte = new string[bytes.Length];

            for (int i = 0; i < bytes.Length; ++i)
                Byte[i] = $"{bytes[i]:X2}";
        }
    }

    /// <summary>
    /// Interaction logic for MemoryView.xaml
    /// </summary>
    public partial class MemoryView : UserControl
    {
        const uint COLUMNS = 32;

        public MemoryView()
        {
            InitializeComponent();
            Data.Columns.Clear();
            Data.Columns.Add(new DataGridTextColumn() { Binding = new Binding("Address"), Header = "Address", Width = DataGridLength.Auto });
            for (uint i = 0; i < COLUMNS; ++i)
                Data.Columns.Add(new DataGridTextColumn() { Binding = new Binding($"Byte[{i}]"), Header = $"{i:X2}", Width = DataGridLength.Auto });
        }

        public void UpdateData(Data.Page page, uint page_id)
        {
            Data.Items.Clear();
            if (page.data == null)
                Data.IsEnabled = false;
            else
            {
                Data.IsEnabled = true;
                for (uint i = 0; i < page.data.Length; i += COLUMNS)
                {
                    byte[] bytes = new byte[COLUMNS];
                    for (uint j = 0; j < COLUMNS; j++)
                        bytes[j] = page.data[i + j];

                    Data.Items.Add(new MemoryViewRow((page_id << 16) | i, bytes));
                }
            }
        }
    }
}
