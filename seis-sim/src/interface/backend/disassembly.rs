use serde::Serialize;

#[derive(Serialize)]
pub struct DisassemblyRow {
    pub address: String,
    pub bytes: [String; 4],
    pub instruction: String,
}
