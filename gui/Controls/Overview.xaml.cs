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
    public struct OverviewContent
    {
        public uint clocks;
        public uint cache_hits;
        public uint cache_misses;
        public uint cold_misses;
        public uint conflict_misses;
        public uint memory_accesses;
        public uint evictions;
    }
    /// <summary>
    /// Interaction logic for Overview.xaml
    /// </summary>
    public partial class Overview : UserControl
    {
        public Overview()
        {
            InitializeComponent();
        }

        public void UpdateData(OverviewContent content)
        {
            Clocks.Content = content.clocks.ToString();
            Hits.Content = content.cache_hits.ToString();
            Misses.Content = content.cache_misses.ToString();
            Cold.Content = content.cold_misses.ToString();
            Conflict.Content = content.conflict_misses.ToString();
            Accesses.Content = content.memory_accesses.ToString();
            Evictions.Content = content.evictions.ToString();
        }
    }
}
