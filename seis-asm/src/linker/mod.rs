mod constants;
pub mod error;
mod labels;

use self::{constants::Constant, error::Error};
use crate::{
    linker::labels::Label,
    parse::{Instruction, Lines, Span},
};
use libseis::{
    instruction_set::Encode,
    pages::{PAGE_SIZE, STACK_PAGE, ZERO_PAGE},
    types::{Byte, SWord, Short, Word},
};
use std::{
    collections::{HashMap, LinkedList},
    mem::transmute,
    path::Path,
};

macro_rules! byte_len {
    ($container:ident) => {
        ($container.len() * std::mem::size_of_val($container.first().unwrap()))
    };
}

pub struct Page {
    data: [u8; PAGE_SIZE],
    len: Word,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            data: [0; PAGE_SIZE],
            len: 0,
        }
    }
}

impl Page {
    pub fn read(&self, byte: Word) -> u8 {
        self.data[(byte & 0xFFFF) as usize]
    }

    pub fn write(&mut self, byte: Word, value: u8) {
        let addr = byte & 0xFFFF;
        if addr >= self.len {
            self.len = addr + 1;
        }
        self.data[byte as usize] = value;
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }
}

pub struct PageSet(HashMap<Short, Page>);

impl PageSet {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    fn page(&mut self, page_id: Short) -> &mut Page {
        if !self.0.contains_key(&page_id) {
            self.0.insert(page_id, Default::default());
        }

        self.0.get_mut(&page_id).unwrap()
    }

    fn page_of(&mut self, address: Word) -> &mut Page {
        let page_id = (address >> 16) as Short;
        self.page(page_id)
    }

    pub fn write<A: AsRef<Path>>(self, destination: A) -> std::io::Result<()> {
        todo!()
    }
}

pub fn link_symbols(lines: Lines) -> Result<PageSet, Error> {
    let mut pages: PageSet = PageSet::new();
    let mut constants: HashMap<String, Constant> = HashMap::new();
    let mut non_const = Lines::new();

    // Step 1: resolve constants

    for line in lines.into_iter() {
        use crate::parse::LineType as T;
        match line {
            T::Constant(value, span) => {
                use crate::parse::ConstantValue as T;
                if let Some(constant) = constants.get(&value.ident) {
                    return Err(Error::ExistingConstant {
                        name: value.ident,
                        first: constant.span.clone(),
                        repeat: span,
                    });
                }

                let bits = match &value.value {
                    T::Integer(int) => {
                        let mut max = 0;
                        for i in 1..32 {
                            if *int & (1 << i) != 0 {
                                max = i
                            }
                        }
                        max
                    }
                    T::Float(_) => 32,
                };

                constants.insert(
                    value.ident,
                    Constant {
                        span,
                        value: value.value,
                        bits,
                    },
                );
            }

            x => non_const.push_back(x),
        }
    }

    let mut labels: HashMap<String, Label> = HashMap::new();
    let mut expanded = LinkedList::<(Instruction, Word, Span)>::new();
    let mut data = LinkedList::<(Vec<Byte>, Word, Span)>::new();

    let mut ip = 0;

    // Step 2: resolve labels, expand LOAD instructions (as they're the only instructions that are expanded by the assembler).

    for line in non_const {
        use crate::parse::LineType as T;

        println!("{line:#?}");

        match line {
            T::Instruction(value, span) => match value {
                Instruction::Load(l) => {
                    use crate::parse::ExpandableLoadOp as E;
                    use crate::parse::ImmediateLoadOp as L;
                    match l {
                        E::Integer { value, destination } => {
                            let left = ((value & 0xFFFF_0000) >> 16) as Short;
                            let right = (value & 0x0000_FFFF) as Short;
                            expanded.push_back((
                                Instruction::Ldr(L::Immediate {
                                    value: right,
                                    destination,
                                    location: 0,
                                    insert: false,
                                }),
                                ip,
                                span.clone(),
                            ));
                            if left == 0 {
                                ip += 1;
                            } else {
                                expanded.push_back((
                                    Instruction::Ldr(L::Immediate {
                                        value: left,
                                        destination,
                                        location: 1,
                                        insert: true,
                                    }),
                                    ip + 1,
                                    span,
                                ));
                                ip += 2;
                            }
                        }
                        E::Float { value, destination } => {
                            let value: Word = unsafe { transmute(value) };
                            let left = ((value & 0xFFFF_0000) >> 16) as Short;
                            let right = (value & 0x0000_FFFF) as Short;
                            expanded.push_back((
                                Instruction::Ldr(L::Immediate {
                                    value: right,
                                    destination,
                                    location: 0,
                                    insert: false,
                                }),
                                ip,
                                span.clone(),
                            ));
                            if left == 0 {
                                ip += 1;
                            } else {
                                expanded.push_back((
                                    Instruction::Ldr(L::Immediate {
                                        value: left,
                                        destination,
                                        location: 1,
                                        insert: true,
                                    }),
                                    ip,
                                    span,
                                ));
                                ip += 2;
                            }
                        }
                        E::ConstantVal { ident, destination } => {
                            if let Some(value) = constants.get(&ident) {
                                use crate::parse::ConstantValue as T;
                                let value = match value.value {
                                    T::Integer(int) => int,
                                    T::Float(float) => unsafe { transmute(float) },
                                };

                                let left = ((value & 0xFFFF_0000) >> 16) as Short;
                                let right = (value & 0x0000_FFFF) as Short;
                                expanded.push_back((
                                    Instruction::Ldr(L::Immediate {
                                        value: right,
                                        destination,
                                        location: 0,
                                        insert: false,
                                    }),
                                    ip,
                                    span.clone(),
                                ));
                                if left == 0 {
                                    ip += 1;
                                } else {
                                    expanded.push_back((
                                        Instruction::Ldr(L::Immediate {
                                            value: left,
                                            destination,
                                            location: 1,
                                            insert: true,
                                        }),
                                        ip + 1,
                                        span,
                                    ));
                                    ip += 2;
                                }
                            }
                        }
                        label => {
                            expanded.push_back((Instruction::Load(label), ip, span));
                            ip += 2;
                        }
                    }
                }
                x => {
                    expanded.push_back((x, ip, span));
                    ip += 1;
                }
            },

            T::Directive(value, span) => {
                // TODO: put an assertion to ensure that the address does not go past the memory upper-bound.
                use crate::parse::Directive::*;
                match &value {
                    &Location(address) => {
                        if address & 0xFFFF_0000 == STACK_PAGE {
                            return Err(Error::WritingToStack { span });
                        }
                        if address & 0xFFFF_0000 == ZERO_PAGE {
                            return Err(Error::WritingToZeroPage { span });
                        }

                        if pages.page_of(address).len >= address & 0x0000_FFFF {
                            println!("At {span}:\nBy writing to address {address:#x}, you may potentially be overwriting code or data. It is recommended that you move the code or data elsewhere.")
                        }

                        ip = address;
                    }
                }
            }

            T::Label(name, span) => {
                if let Some(label) = labels.get(&name) {
                    return Err(Error::ExistingLabel {
                        name,
                        first: label.span.clone(),
                        repeat: span,
                    });
                }

                labels.insert(name, Label { address: ip, span });
            }

            T::Data(content, span) => {
                use crate::parse::Data as D;

                match content {
                    D::Byte(bytes) => {
                        let len = byte_len!(bytes) as Word;
                        data.push_back((bytes, ip, span));
                        ip += len;
                    }
                    D::Short(shorts) => {
                        let len = byte_len!(shorts) as Word;
                        let bytes = shorts.into_iter().map(Short::to_be_bytes).fold(
                            vec![],
                            |mut acc, e| {
                                acc.extend_from_slice(&e);
                                acc
                            },
                        );
                        data.push_back((bytes, ip, span));
                        ip += len;
                    }
                    D::Word(words) => {
                        let len = byte_len!(words) as Word;
                        let bytes =
                            words
                                .into_iter()
                                .map(Word::to_be_bytes)
                                .fold(vec![], |mut acc, e| {
                                    acc.extend_from_slice(&e);
                                    acc
                                });
                        data.push_back((bytes, ip, span));
                        ip += len;
                    }
                    D::Float(floats) => {
                        let len = byte_len!(floats) as Word;
                        let bytes =
                            floats
                                .into_iter()
                                .map(f32::to_be_bytes)
                                .fold(vec![], |mut acc, e| {
                                    acc.extend_from_slice(&e);
                                    acc
                                });
                        data.push_back((bytes, ip, span));
                        ip += len;
                    }
                    D::String(strings) => {
                        let mut bytes = vec![];
                        for string in strings {
                            let mut chars = string.chars();

                            while let Some(char) = chars.next() {
                                if char == '\\' {
                                    let result = match chars.next().unwrap() {
                                        'n' => '\n',
                                        'r' => '\r',
                                        't' => '\t',
                                        x => x,
                                    };

                                    let mut buf = [0; 4];
                                    bytes.extend_from_slice(result.encode_utf8(&mut buf).as_bytes())
                                } else {
                                    let mut buf = [0; 4];
                                    bytes.extend_from_slice(char.encode_utf8(&mut buf).as_bytes())
                                }
                            }

                            bytes.push(0);
                        }
                        let len = bytes.len() as Word;
                        data.push_back((bytes, ip, span));
                        ip += len;
                    }
                }
            }

            _ => unreachable!("constants are all removed from the list"),
        }
    }

    // Stage 3: transform into instructions

    for (instruction, address, span) in expanded {
        use crate::parse::{
            ExpandableLoadOp::Label, Instruction as I, IntBinaryOp as IBO, IntCompOp as ICO,
            Jump as J,
        };
        use libseis::instruction_set::{
            control::Jump, integer, ControlOp::*, FloatingPointOp::*, Instruction::*, IntegerOp::*,
            RegisterOp::*,
        };

        let write = |(b, a)| pages.page_of(a).write(a, b);

        let transform_jump = |j| match j {
            J::Absolute(reg) => Ok(Jump::Register(reg)),
            J::Relative(rel) => Ok(Jump::Relative(rel as SWord)),
            J::Label(label_name) => {
                if let Some(label) = labels.get(&label_name) {
                    let laddr = label.address;
                    let dist = (address - laddr) as SWord;
                    if dist > -8_388_608 || dist < 8_388_607 {
                        Ok(Jump::Relative(dist))
                    } else {
                        Err(Error::JumpTooLong {
                            label: label_name,
                            span: span.clone(),
                        })
                    }
                } else {
                    Err(Error::NonExistingLabel {
                        name: label_name,
                        usage: span.clone(),
                    })
                }
            }
        };

        let transform_int_bop = |b| match b {
            IBO::RegReg {
                source,
                opt,
                destination,
            } => Ok(integer::BinaryOp::Registers(source, opt, destination)),
            IBO::RegImm {
                source,
                opt,
                destination,
            } => Ok(integer::BinaryOp::Immediate(source, opt, destination)),
            IBO::RegConst {
                source,
                opt,
                destination,
            } => {
                if let Some(optval) = constants.get(&opt) {
                    use crate::parse::ConstantValue as T;
                    let value = match optval.value {
                        T::Integer(i) => i,
                        T::Float(_) => {
                            return Err(Error::IntTypeMismatch {
                                name: opt,
                                span: span.clone(),
                            })
                        }
                    };

                    if optval.bits > 15 {
                        Err(Error::ConstTooLong {
                            name: opt,
                            span: span.clone(),
                        })
                    } else {
                        Ok(integer::BinaryOp::Immediate(source, value, destination))
                    }
                } else {
                    Err(Error::NonExistingConstant {
                        name: opt,
                        usage: span.clone(),
                    })
                }
            }
        };

        let transform_int_cop = |c| match c {
            ICO::RegReg { left, right } => Ok(integer::CompOp::Registers(left, right)),
            ICO::RegImm { left, right } => Ok(integer::CompOp::Immediate(left, right)),
            ICO::RegConst { left, right } => todo!(),
        };

        match instruction {
            I::Halt => Control(Halt)
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Nop => Control(Nop)
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jmp(j) => Control(Jmp(transform_jump(j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jsr(j) => Control(Jsr(transform_jump(j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jeq(j) => Control(Jeq(transform_jump(j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jne(j) => Control(Jne(transform_jump(j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jgt(j) => Control(Jgt(transform_jump(j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jlt(j) => Control(Jlt(transform_jump(j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jge(j) => Control(Jge(transform_jump(j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jle(j) => Control(Jle(transform_jump(j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Ret => Control(Ret)
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),

            I::Add(b) => Integer(Add(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Sub(b) => Integer(Sub(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Mul(b) => Integer(Mul(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Dvu(b) => Integer(Dvu(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Dvs(b) => Integer(Dvs(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Mod(b) => Integer(Mod(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::And(b) => Integer(And(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Ior(b) => Integer(Ior(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Xor(b) => Integer(Xor(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Bsl(b) => Integer(Bsl(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Bsr(b) => Integer(Bsr(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Asr(b) => Integer(Asr(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Rol(b) => Integer(Rol(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Ror(b) => Integer(Ror(transform_int_bop(b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Not(u) => Integer(Not(integer::UnaryOp(u.source, u.destination)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Sxt(s) => Integer(Sxt(integer::SignExtentOp(s.width, s.register)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Cmp(c) => Integer(Cmp(transform_int_cop(c)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Tst(c) => Integer(Tst(transform_int_cop(c)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),

            I::Fadd(_) => todo!(),
            I::Fsub(_) => todo!(),
            I::Fmul(_) => todo!(),
            I::Fdiv(_) => todo!(),
            I::Fmod(_) => todo!(),
            I::Fcmp(_) => todo!(),
            I::Fneg(_) => todo!(),
            I::Frec(_) => todo!(),
            I::Itof(_) => todo!(),
            I::Ftoi(_) => todo!(),
            I::Fchk(_) => todo!(),

            I::Push(_) => todo!(),
            I::Pop(_) => todo!(),
            I::Lbr(_) => todo!(),
            I::Sbr(_) => todo!(),
            I::Lsr(_) => todo!(),
            I::Ssr(_) => todo!(),
            I::Llr(_) => todo!(),
            I::Slr(_) => todo!(),
            I::Tfr(_, _) => todo!(),
            I::Ldr(_) => todo!(),

            I::Load(Label { ident, destination }) => todo!(),

            _ => unreachable!("Non-label LOAD instructions have already been evaluated."),
        }
    }

    Ok(pages)
}
