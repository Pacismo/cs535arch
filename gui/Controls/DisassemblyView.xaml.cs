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
    public struct DisassemblyViewRow
    {
        public string Address { get; set; }
        public string[] Bytes { get; set; }
        public string Instruction { get; set; }
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

        public void UpdateData(DisassemblyViewRow[]? rows)
        {
            Data.Items.Clear();

            IsEnabled = true;
            if (rows != null)
                foreach (var row in rows)
                    Data.Items.Add(row);
            else
                IsEnabled = false;
        }
    }
}
