use crate::types::Register;

pub const V: [Register; 16] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
];

pub const SP: Register = 0x10;
pub const BP: Register = 0x11;
pub const LP: Register = 0x12;
pub const PC: Register = 0x13;
pub const ZF: Register = 0x14;
pub const OF: Register = 0x14;
pub const EPS: Register = 0x15;
pub const NAN: Register = 0x16;
pub const INF: Register = 0x17;

pub const COUNT: usize = 25;

pub const NAME: [&'static str; COUNT] = [
    "V0", "V1", "V2", "V3", "V4", "V5", "V6", "V7", "V8", "V9", "VA", "VB", "VC", "VD", "VE", "VF",
    "SP", "BP", "LP", "PC", "ZF", "OF", "EPS", "NAN", "INF",
];

pub const fn get_name(reg: Register) -> Option<&'static str> {
    if (reg as usize) >= COUNT {
        None
    } else {
        Some(NAME[reg as usize])
    }
}

pub fn get_id(name: &str) -> Option<Register> {
    let target = name.to_uppercase();

    NAME.into_iter()
        .enumerate()
        .find(|&(_, s)| s == target)
        .map(|(i, _)| i as Register)
}
