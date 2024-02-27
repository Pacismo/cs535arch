/// One byte \[0, 255]
pub type Byte = u8;
/// Two bytes \[0, 65535]
pub type Short = u16;
/// Four bytes \[0, 4294967295]
pub type Long = u32;
/// The width of an instruction (an alias for a [`Long`])
pub type Word = Long;
/// Represents a register index.
pub type Register = Word;
