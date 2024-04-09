use libseis::types::Word;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DisassemblyRow {
    pub address: String,
    pub bytes: [String; 4],
    pub instruction: String,
}

#[derive(Debug, Serialize)]
pub struct DisassemblyData {
    pub pc: Word,
    pub rows: Vec<DisassemblyRow>,
}
