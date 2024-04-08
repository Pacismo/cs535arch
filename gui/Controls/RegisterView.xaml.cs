using gui.Data;
using System.Windows;
using System.Windows.Controls;

namespace gui.Controls
{
    public struct RegisterViewRow(string register, uint value)
    {
        public string Register { get; set; } = register;
        public string Binary { get; set; } = value.ToString("b32");
        public string Decimal { get; set; } = value.ToString("d");

        public string Hexdecimal { get; set; } = value.ToString("X8");
    }
    /// <summary>
    /// Interaction logic for RegisterView.xaml
    /// </summary>
    public partial class RegisterView : UserControl
    {
        Registers registers;

        public RegisterView()
        {
            InitializeComponent();
        }

        public void UpdateData(Registers registers)
        {
            this.registers = registers;

            Refresh();
        }

        public void Refresh()
        {
            Data.Items.Clear();
            Data.Items.Add(new RegisterViewRow("V0", registers.v0));
            Data.Items.Add(new RegisterViewRow("V1", registers.v1));
            Data.Items.Add(new RegisterViewRow("V2", registers.v2));
            Data.Items.Add(new RegisterViewRow("V3", registers.v3));
            Data.Items.Add(new RegisterViewRow("V4", registers.v4));
            Data.Items.Add(new RegisterViewRow("V5", registers.v5));
            Data.Items.Add(new RegisterViewRow("V6", registers.v6));
            Data.Items.Add(new RegisterViewRow("V7", registers.v7));
            Data.Items.Add(new RegisterViewRow("V8", registers.v8));
            Data.Items.Add(new RegisterViewRow("V9", registers.v9));
            Data.Items.Add(new RegisterViewRow("VA", registers.va));
            Data.Items.Add(new RegisterViewRow("VB", registers.vb));
            Data.Items.Add(new RegisterViewRow("VC", registers.vc));
            Data.Items.Add(new RegisterViewRow("VD", registers.vd));
            Data.Items.Add(new RegisterViewRow("VE", registers.ve));
            Data.Items.Add(new RegisterViewRow("VF", registers.vf));
            Data.Items.Add(new RegisterViewRow("SP", registers.sp));
            Data.Items.Add(new RegisterViewRow("BP", registers.bp));
            Data.Items.Add(new RegisterViewRow("LP", registers.lp));
            Data.Items.Add(new RegisterViewRow("PC", registers.pc));
            Data.Items.Add(new RegisterViewRow("ZF", registers.zf));
            Data.Items.Add(new RegisterViewRow("OF", registers.of));
            Data.Items.Add(new RegisterViewRow("EPS", registers.eps));
            Data.Items.Add(new RegisterViewRow("NAN", registers.nan));
            Data.Items.Add(new RegisterViewRow("INF", registers.inf));
        }

        private void OnLoad(object sender, RoutedEventArgs e)
        {
            if (Data.Items.Count > 0) return;
            Refresh();
        }
    }
}
