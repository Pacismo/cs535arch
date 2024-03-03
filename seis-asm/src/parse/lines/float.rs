use libseis::types::Register;

#[derive(Debug)]
pub struct FloatBinaryOp {
    pub source: Register,
    pub opt: Register,
    pub destination: Register,
}

#[derive(Debug)]
pub struct FloatUnaryOp {
    pub source: Register,
    pub destination: Register,
}

#[derive(Debug)]
pub struct FloatCompOp {
    pub left: Register,
    pub right: Register,
}
