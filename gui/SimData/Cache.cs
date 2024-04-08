using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Newtonsoft.Json;

namespace gui.Data
{
    public struct Line
    {
        public uint base_address;
        public bool dirty;
        public byte[] data;

        public byte this[int i] { get { return data[i]; } }
    }

    public struct Cache
    {
        public string name;
        public Line?[] lines;

        public Line? this[int i] { get { return lines[i]; } }
    }
}
