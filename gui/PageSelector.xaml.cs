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
using System.Windows.Shapes;

namespace gui
{
    /// <summary>
    /// Interaction logic for PageSelector.xaml
    /// </summary>
    public partial class PageSelector : Window
    {
        public static RoutedCommand OkCommand = new();
        public static RoutedCommand CancelCommand = new();

        uint? page_id;
        uint max;

        public PageSelector(uint old_pid, uint max_page)
        {
            InitializeComponent();
            page_id = old_pid;
            max = max_page;
        }

        private void PageID_TextChanged(object sender, TextChangedEventArgs e)
        {
            try { 
                page_id = uint.Parse(PageID.Text);
                if (page_id < 0 || page_id >= max)
                {
                    page_id = null;
                    PageID.Background = new SolidColorBrush(Colors.Red);
                    PageID.Foreground = new SolidColorBrush(Colors.White);
                    return;
                }
            }
            catch
            {
                page_id = null;
                PageID.Background = new SolidColorBrush(Colors.OrangeRed);
                return;
            }
            PageID.Background = new SolidColorBrush(Colors.White);
            PageID.Foreground = new SolidColorBrush(Colors.Black);
        }

        public uint? GetPageID() => page_id;

        private void OkCommandHandler(object sender, ExecutedRoutedEventArgs e)
        {
            if (page_id != null)
                Close();
            else
                new OkDialog("Error", $"Please enter a valid integer between 0 and {max}").ShowDialog();
        }

        private void CancelCommandHandler(object sender, ExecutedRoutedEventArgs e)
        {
            page_id = null;
            Close();
        }
    }
}
