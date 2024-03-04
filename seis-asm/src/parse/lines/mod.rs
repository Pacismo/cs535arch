mod float;
mod integer;
mod register;

pub use float::*;
pub use integer::*;
use libseis::types::{Byte, Register, Short, Word};
pub use register::*;
use std::{
    collections::LinkedList,
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Span {
    pub file: PathBuf,
    pub line: u64,
}

impl Span {
    pub fn new(file: &Path, line: u64) -> Self {
        Self {
            file: file.to_owned(),
            line,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}",
            self.file.file_name().unwrap().to_string_lossy(),
            self.line
        )
    }
}

#[derive(Debug)]
pub struct Lines(pub(super) LinkedList<LineType>);

impl std::ops::DerefMut for Lines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Lines {
    type Target = LinkedList<LineType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Lines {
    pub fn new() -> Self {
        Self(LinkedList::new())
    }
}

#[derive(Debug)]
pub enum ConstantValue {
    Byte(Byte),
    Short(Short),
    Word(Word),
    Float(f32),
    String(String),
}

#[derive(Debug)]
pub struct Constant {
    pub ident: String,
    pub value: ConstantValue,
}

#[derive(Debug)]
pub enum Directive {
    Location(Word),
    Public,
    ZeroPage,
}

#[derive(Debug)]
pub enum Jump {
    Relative(Word),
    Absolute(Register),
    Label(String),
}

#[derive(Debug)]
pub enum LineType {
    Constant(Constant, Span),
    Instruction(Instruction, Span),
    Directive(Directive, Span),
    Label(String),
}

#[derive(Debug)]
pub enum Instruction {
    Halt,
    Nop,
    Jmp(Jump),
    Jsr(Jump),
    Jeq(Jump),
    Jne(Jump),
    Jgt(Jump),
    Jlt(Jump),
    Jge(Jump),
    Jle(Jump),
    Ret,

    Add(IntBinaryOp),
    Sub(IntBinaryOp),
    Mul(IntBinaryOp),
    Dvu(IntBinaryOp),
    Dvs(IntBinaryOp),
    Mod(IntBinaryOp),
    And(IntBinaryOp),
    Ior(IntBinaryOp),
    Xor(IntBinaryOp),
    Not(IntUnaryOp),
    Sxt(IntSignExtendOp),
    Bsl(IntBinaryOp),
    Bsr(IntBinaryOp),
    Asr(IntBinaryOp),
    Rol(IntBinaryOp),
    Ror(IntBinaryOp),
    Cmp(IntCompOp),
    Tst(IntCompOp),

    Fadd(FloatBinaryOp),
    Fsub(FloatBinaryOp),
    Fmul(FloatBinaryOp),
    Fdiv(FloatBinaryOp),
    Fmod(FloatBinaryOp),
    Fcmp(FloatCompOp),
    Fneg(FloatUnaryOp),
    Frec(FloatUnaryOp),
    Itof(FloatUnaryOp),
    Ftoi(FloatUnaryOp),
    Fchk(Register),

    Push(StackOp),
    Pop(StackOp),
    Lbr(MemoryLoadOp),
    Sbr(MemoryLoadOp),
    Lsr(MemoryLoadOp),
    Ssr(MemoryLoadOp),
    Llr(MemoryLoadOp),
    Slr(MemoryLoadOp),
    Tfr(Register, Register),
    Ldr(ImmediateLoadOp),
    Load(ExpandableLoadOp),
}
