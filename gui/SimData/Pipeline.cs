using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Nodes;
using System.Threading.Tasks;

namespace gui.Data
{
    public readonly struct Pipeline
    {
        public readonly JsonObject fetch;
        public readonly JsonObject decode;
        public readonly JsonObject execute;
        public readonly JsonObject memory;
        public readonly JsonObject writeback;
    }
}
