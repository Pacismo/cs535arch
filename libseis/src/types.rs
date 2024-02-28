/// One byte \[0, 255]
pub type Byte = u8;
/// One byte \[-128, 127]
pub type SByte = i8;
/// Two bytes \[0, 65535]
pub type Short = u16;
/// Two bytes \[-32768. 32767]
pub type SShort = i16;
/// Four bytes \[0, 4294967295]
pub type Long = u32;
/// Four bytes \[-2147483648, 2147483647]
pub type SLong = i32;
/// The width of an instruction (an alias for a [`Long`])
pub type Word = Long;
/// The width of an instruction (an alias for a [`SLong`])
pub type SWord = SLong;
/// Represents a register index.
pub type Register = Byte;
