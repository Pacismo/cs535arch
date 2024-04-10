using System.Windows;
using System.Windows.Controls;

namespace gui.Controls
{
    public struct PcOnlyRegisterStruct
    {
        public uint pc;
    }

    public struct DisassembledWord
    {
        public string address;
        public string[] bytes;
        public string instruction;
    }

    public struct DisassemblyViewRow(DisassembledWord word)
    {
        public bool? Current { get; set; } = false;
        public string Address { get; set; } = word.address;
        public string[] Bytes { get; set; } = word.bytes;
        public string Instruction { get; set; } = word.instruction;
    }

    /// <summary>
    /// Interaction logic for DisassemblyView.xaml
    /// </summary>
    public partial class DisassemblyView : UserControl
    {
        uint current_page = 0;
        uint pc = 0;
        public uint PC
        {
            get { return pc; }
            set
            {
                if (pc == value)
                    return;

                uint old_pc = pc;
                pc = value;

                if ((old_pc & 0xFFFF_0000) == (current_page << 16))
                {
                    int i = (int) (old_pc & 0xFFFF) >> 2;
                    var row = Data.Items[i] as DisassemblyViewRow?
                        ?? throw new InvalidCastException("The data is expected to be of type DisassemblyViewRow! HOW?!");
                    row.Current = false;

                    // Data.Items.RemoveAt(i);
                    // Data.Items.Insert(i, row);
                    Data.Items[i] = row;
                }
                if ((pc & 0xFFFF_0000) == (current_page << 16))
                {
                    int i = (int) (pc & 0xFFFF) >> 2;
                    var row = Data.Items[i] as DisassemblyViewRow?
                        ?? throw new InvalidCastException("The data is expected to be of type DisassemblyViewRow! HOW?!");
                    row.Current = null;

                    // Data.Items.RemoveAt(i);
                    // Data.Items.Insert(i, row);
                    Data.Items[i] = row;
                }
            }
        }

        public DisassemblyView()
        {
            InitializeComponent();
        }

        public void UpdateData(uint current_page, DisassembledWord[]? data, uint? pc = 0)
        {
            Data.Items.Clear();
            if (pc.HasValue)
                this.pc = pc.Value;

            IsEnabled = true;
            if (data != null)
                for (uint i = 0; i < data.Length; ++i)
                {
                    DisassemblyViewRow row = new(data[i]);
                    if (i << 2 == (pc & 0xFFFF) && pc >> 16 == current_page)
                        row.Current = null;
                    Data.Items.Add(row);
                }
            else
                IsEnabled = false;

            this.current_page = current_page;
        }
    }
}
