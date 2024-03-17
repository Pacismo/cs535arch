using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;

namespace gui.SimConfig
{
    internal class InvalidValueException(string desc) : Exception(desc) { }

    /// <summary>
    /// Determines the mode that the cache operates in
    /// </summary>
    public enum CacheType
    {
        /// <summary>
        /// No cache is present
        /// </summary>
        None,
        /// <summary>
        /// An associative cache is used
        /// </summary>
        Associative
    }

    /// <summary>
    /// Represents a configuration to use when running a program
    /// </summary>
    public struct CacheConfiguration(CacheType type = CacheType.None, uint? set_bits = null, uint? byte_bits = null, uint? ways = null)
    {
        public CacheType type = type;
        /// <summary>
        /// The number of set bits to use
        /// </summary>
        public uint? set_bits = set_bits;
        /// <summary>
        /// The number of byte offset bits to use
        /// </summary>
        public uint? byte_bits = byte_bits;
        /// <summary>
        /// The number of lines per cache set
        /// </summary>
        public uint? ways = ways;
    }
}
