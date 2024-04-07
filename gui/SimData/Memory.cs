using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace gui.Data
{
    public struct Page
    {
        public byte[] data;

        public byte this[uint i]
        {
            readonly get
            {
                return data != null ? data[i] : (byte) 0;
            }
            set
            {
                data ??= new byte[65536];
                data[i] = value;
            }
        }

        public struct Memory
        {
            public Page[] pages;

            public readonly byte this[uint i]
            {
                get
                {
                    uint page_id = (i & 0xFFFF_0000) >> 16;
                    uint address = i & 0xFFFF;
                    return pages[page_id][address];
                }
                set
                {
                    uint page_id = (i & 0xFFFF_0000) >> 16;
                    uint address = i & 0xFFFF;
                    pages[page_id][address] = value;
                }
            }
        }
    }
}
