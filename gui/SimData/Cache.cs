using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Newtonsoft.Json;

namespace seisgui
{
    struct Line
    {
        public uint base_address;
        public bool dirty;
        public byte[] data;

        public byte this[int i] { get { return data[i]; } }
    }

    internal readonly struct Cache
    {
        readonly string name;
        readonly Line[] lines;

        public string Name { get { return name; } }
        public Line this[int i] { get { return lines[i]; } }
    }
}
