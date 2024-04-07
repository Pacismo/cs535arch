using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Newtonsoft.Json;

namespace gui.Data
{
    public readonly struct Line
    {
        public readonly uint base_address;
        public readonly bool dirty;
        public readonly byte[] data;

        public byte this[int i] { get { return data[i]; } }
    }

    public readonly struct Cache
    {
        public readonly string name;
        public readonly Line[] lines;

        public Line this[int i] { get { return lines[i]; } }
    }
}
