using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace gui.Data
{
    public readonly struct Instruction
    {
        public readonly string decoded;

        public static implicit operator string(Instruction i) { return i.decoded; }
    }
}
