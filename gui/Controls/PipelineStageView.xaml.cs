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
    struct PipelineStageDataRow
    {
        public string Name { get; set; }
        public string Value { get; set; }
    }

    /// <summary>
    /// Interaction logic for PipelineStageView.xaml
    /// </summary>
    public partial class PipelineStageView : UserControl
    {
        public string Header { get; set; } = "Pipeline Stage";
        public PipelineStageView()
        {
            InitializeComponent();
            DataContext = this;
        }

        public void UpdateData(Dictionary<string, object> data)
        {
            Data.Items.Clear();
            if (data != null)
                foreach (var pair in data)
                    Data.Items.Add(new PipelineStageDataRow() { Name = pair.Key, Value = pair.Value?.ToString() ?? "<?>" });

            foreach (var column in Data.Columns)
                column.Width = DataGridLength.SizeToCells;
        }
    }
}
