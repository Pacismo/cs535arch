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
    public struct DisassemblyViewData
    {
        public uint pc;
        public DisassembledWord[]? rows;
    }

    public struct DisassembledWord
    {
        public string address;
        public string[] bytes;
        public string instruction;
    }

    public struct DisassemblyViewRow(DisassembledWord word, bool is_current)
    {
        public bool Current { get; set; } = is_current;
        public string Address { get; set; } = word.address;
        public string[] Bytes { get; set; } = word.bytes;
        public string Instruction { get; set; } = word.instruction;
    }

    /// <summary>
    /// Interaction logic for DisassemblyView.xaml
    /// </summary>
    public partial class DisassemblyView : UserControl
    {
        public DisassemblyView()
        {
            InitializeComponent();
        }

        public void UpdateData(uint current_page, DisassemblyViewData data)
        {
            Data.Items.Clear();

            IsEnabled = true;
            if (data.rows != null)
            {
                for (uint i = 0; i < data.rows.Length; ++i)
                    Data.Items.Add(new DisassemblyViewRow(data.rows[i], (i * 4 | current_page << 16) == data.pc));
            }
            else
                IsEnabled = false;
        }
    }
}
