use libseis::types::{Register, Word};

#[derive(Debug)]
pub enum IntBinaryOp {
    RegReg {
        source: Register,
        opt: Register,
        destination: Register,
    },
    RegImm {
        source: Register,
        opt: Word,
        destination: Register,
    },
    RegConst {
        source: Register,
        opt: String,
        destination: Register,
    },
}

#[derive(Debug)]
pub struct IntUnaryOp {
    pub source: Register,
    pub destination: Register,
}

#[derive(Debug)]
pub enum IntCompOp {
    RegReg {
        left: Register,
        right: Register,
        signed: bool,
    },
    RegImm {
        left: Register,
        right: Word,
        signed: bool,
    },
    RegConst {
        left: Register,
        right: String,
        signed: bool,
    },
}

#[derive(Debug)]
pub enum IntTestOp {
    RegReg { left: Register, right: Register },
    RegImm { left: Register, right: Word },
    RegConst { left: Register, right: String },
}

#[derive(Debug)]
pub struct IntSignExtendOp {
    pub register: Register,
    pub width: Word,
}
