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
using System.Windows.Media.Imaging;
using System.Windows.Shapes;

namespace gui.SimConfig
{
    /// <summary>
    /// Interaction logic for Configurator.xaml
    /// </summary>
    public partial class Configurator : Window
    {
        CacheConfiguration cache_config;

        public CacheConfiguration CacheConfiguration { get { return cache_config; } }

        public Configurator()
        {
            InitializeComponent();
        }

        private void Okay_Click(object sender, RoutedEventArgs e)
        {
            if (cache_enable.IsChecked == true)
            {
                cache_config = new CacheConfiguration(
                    CacheType.Associative,
                    (uint) set_bits_slider.Value,
                    (uint) byte_offset_slider.Value,
                    (uint) Math.Pow(2, ways_slider.Value)
                );
            }
            DialogResult = true;
            Close();
        }

        private void Cancel_Click(object sender, RoutedEventArgs e)
        {
            DialogResult = false;
            Close();
        }

        private void ways_slider_ValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            if (ways_label == null) return;
            ways_label.Content = Math.Floor(Math.Pow(2, ways_slider.Value)).ToString();
        }

        private void set_bits_slider_ValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            if (sets_label == null) return;
            sets_label.Content = Math.Floor(set_bits_slider.Value).ToString();
            if (set_bits_slider.Value + byte_offset_slider.Value > 32)
                byte_offset_slider.Value = 32 - set_bits_slider.Value;
        }

        private void byte_offset_slider_ValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            if (byte_offset_label == null) return;
            byte_offset_label.Content = Math.Floor(byte_offset_slider.Value).ToString();
            if (set_bits_slider.Value + byte_offset_slider.Value > 32)
                set_bits_slider.Value = 32 - byte_offset_slider.Value;
        }

        private void cache_enable_Checked(object sender, RoutedEventArgs e)
        {
            if (associative_config_group == null) return;

            associative_config_group.IsEnabled = cache_enable.IsChecked == true;
        }
    }
}
