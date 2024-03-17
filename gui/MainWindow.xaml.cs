using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;
using gui.SimConfig;

namespace gui
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        CacheConfiguration cache_config;
        public MainWindow()
        {
            InitializeComponent();
        }

        private void Exit_Click(object sender, RoutedEventArgs e)
        {
            // TODO: Handle the necessary steps to close the window
            Close();
        }

        private void NewSimulation_Click(object sender, RoutedEventArgs e)
        {
            Configurator configurator = new();
            if (configurator.ShowDialog() == true)
                cache_config = configurator.CacheConfiguration;
        }
    }
}