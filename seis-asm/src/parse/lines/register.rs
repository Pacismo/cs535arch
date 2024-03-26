use libseis::types::{Byte, Register, Short};

#[derive(Debug)]
pub enum StackOp {
    Registers(Vec<Register>),
    Register(Register),
}

#[derive(Debug)]
pub enum MemoryLoadOp {
    Zpg {
        address: Short,
        destination: Register,
    },
    ConstZpg {
        constant: String,
        destination: Register,
    },
    Indirect {
        address: Register,
        destination: Register,
        volatile: bool,
    },
    Offset {
        address: Register,
        offset: Short,
        destination: Register,
        volatile: bool,
    },
    Indexed {
        address: Register,
        index: Register,
        destination: Register,
        volatile: bool,
    },
    Stack {
        offset: Short,
        destination: Register,
    },
}

#[derive(Debug)]
pub enum MemoryStoreOp {
    Zpg {
        address: Short,
        source: Register,
    },
    ConstZpg {
        constant: String,
        source: Register,
    },
    Indirect {
        address: Register,
        source: Register,
        volatile: bool,
    },
    Offset {
        address: Register,
        offset: Short,
        source: Register,
        volatile: bool,
    },
    Indexed {
        address: Register,
        index: Register,
        source: Register,
        volatile: bool,
    },
    Stack {
        offset: Short,
        source: Register,
    },
}

#[derive(Debug)]
pub enum ImmediateLoadOp {
    ZpgAddr {
        address: Short,
        destination: Register,
    },
    ConstZpgAddr {
        constant: String,
        destination: Register,
    },
    Immediate {
        value: Short,
        destination: Register,
        location: Byte,
        insert: bool,
    },
}

#[derive(Debug)]
pub enum ExpandableLoadOp {
    Integer {
        value: i64,
        destination: Register,
    },
    Float {
        value: f32,
        destination: Register,
    },
    Label {
        ident: String,
        destination: Register,
    },
    ConstantVal {
        ident: String,
        destination: Register,
    },
}
