use libpipe::Registers;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct RegsFp32 {
    /// Variable registers
    pub v: [f32; 16],
    /// Stack pointer
    pub sp: f32,
    /// Stack base pointer
    pub bp: f32,
    /// Link pointer
    pub lp: f32,
    /// Program counter
    pub pc: f32,
    /// Zero flag
    pub zf: f32,
    /// Overflow flag
    pub of: f32,
    /// Epsilon equality flag
    pub eps: f32,
    /// NaN flag
    pub nan: f32,
    /// Infinity flag
    pub inf: f32,
}

impl From<Registers> for RegsFp32 {
    fn from(value: Registers) -> Self {
        Self {
            v: value.v.map(f32::from_bits),
            sp: f32::from_bits(value.sp),
            bp: f32::from_bits(value.bp),
            lp: f32::from_bits(value.lp),
            pc: f32::from_bits(value.pc),
            zf: f32::from_bits(value.zf),
            of: f32::from_bits(value.of),
            eps: f32::from_bits(value.eps),
            nan: f32::from_bits(value.nan),
            inf: f32::from_bits(value.inf),
        }
    }
}
