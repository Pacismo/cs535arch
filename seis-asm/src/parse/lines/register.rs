use libseis::types::{Byte, Register, Short};

#[derive(Debug)]
pub struct StackOp(pub Vec<Register>);

impl std::ops::Deref for StackOp {
    type Target = Vec<Register>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum MemoryLoadOp {
    Zpg {
        address: Short,
        destination: Register,
    },
    Indirect {
        address: Register,
        destination: Register,
    },
    Offset {
        address: Register,
        offset: Short,
        destination: Register,
    },
    Indexed {
        address: Register,
        index: Register,
        destination: Register,
    },
    Stack {
        offset: Short,
        destination: Register,
    },
}

#[derive(Debug)]
pub enum ImmediateLoadOp {
    ZpgAddr {
        address: Short,
        destination: Register,
    },
    Immediate {
        value: Short,
        destination: Register,
        location: Byte,
        insert: bool,
    },
    Constant {
        ident: String,
        destination: Register,
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
    ConstantRef {
        ident: String,
        destination: Register,
    },
}
