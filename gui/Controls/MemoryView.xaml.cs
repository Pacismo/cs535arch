using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;

namespace gui.Controls
{
    public struct MemoryViewRow
    {
        public string Address { get; set; }
        public string[] Byte { get; set; }

        public MemoryViewRow(uint address, byte[] bytes, bool binary = false)
        {
            Address = $"0x{address:X8}";
            Byte = new string[bytes.Length];

            for (int i = 0; i < bytes.Length; ++i)
            {
                Byte[i] = binary ? $"{bytes[i]:b8}" : $"{bytes[i]:X2}";
            }
        }
    }

    /// <summary>
    /// Interaction logic for MemoryView.xaml
    /// </summary>
    public partial class MemoryView : UserControl
    {
        public MemoryView()
        {
            InitializeComponent();
        }

        public void UpdateData(Data.Page page, uint page_id, bool binary)
        {
            Data.Items.Clear();
            if (page.data == null)
                Data.IsEnabled = false;
            else
            {
                Data.IsEnabled = true;
                for (uint i = 0; i < page.data.Length; i += 16)
                {
                    byte[] bytes = new byte[16];
                    for (uint j = 0; j < 16; j++)
                        bytes[j] = page.data[i + j];

                    Data.Items.Add(new MemoryViewRow((page_id << 16) | i, bytes, binary));
                }

                foreach (var column in Data.Columns)
                    column.Width = DataGridLength.SizeToCells;
            }
        }
    }
}
