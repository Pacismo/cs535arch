mod constants;
pub mod error;
mod labels;

use self::{constants::Constant, error::Error};
use crate::{
    linker::labels::Label,
    parse::{Instruction, Lines, Span, StackOp},
};
use libseis::{
    instruction_set::Encode,
    pages::{PAGE_SIZE, STACK_PAGE, ZERO_PAGE},
    types::{Byte, SWord, Short, Word},
};
use std::{
    collections::{HashMap, LinkedList},
    io::{Seek, Write},
    mem::transmute,
};

macro_rules! byte_len {
    ($container:ident) => {
        ($container.len() * std::mem::size_of_val($container.first().unwrap()))
    };
}

#[derive(Debug)]
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
    pub fn write(&mut self, byte: Word, value: u8) {
        let addr = byte & 0xFFFF;
        if addr >= self.len {
            self.len = addr + 1;
        }
        self.data[byte as usize] = value;
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }
}

#[derive(Debug)]
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

    pub fn write<W: Write + Seek>(self, mut destination: W) -> std::io::Result<()> {
        use std::io::SeekFrom::Start;

        for (page_number, page) in self.0 {
            let base_address = (page_number as u64) << 16;
            destination.seek(Start(base_address))?;
            destination.write(page.data())?;
        }
        Ok(())
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

        match line {
            T::Instruction(value, span) => {
                if ip % 4 != 0 {
                    return Err(Error::MisalignedCode { span });
                }

                match value {
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
                                        zero: false,
                                    }),
                                    ip,
                                    span.clone(),
                                ));
                                ip += 4;
                                if left != 0 {
                                    expanded.push_back((
                                        Instruction::Ldr(L::Immediate {
                                            value: left,
                                            destination,
                                            location: 1,
                                            zero: true,
                                        }),
                                        ip,
                                        span,
                                    ));
                                    ip += 4;
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
                                        zero: false,
                                    }),
                                    ip,
                                    span.clone(),
                                ));
                                if left == 0 {
                                    ip += 4;
                                } else {
                                    expanded.push_back((
                                        Instruction::Ldr(L::Immediate {
                                            value: left,
                                            destination,
                                            location: 1,
                                            zero: true,
                                        }),
                                        ip,
                                        span,
                                    ));
                                    ip += 4;
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
                                            zero: false,
                                        }),
                                        ip,
                                        span.clone(),
                                    ));
                                    ip += 4;
                                    if left != 0 {
                                        expanded.push_back((
                                            Instruction::Ldr(L::Immediate {
                                                value: left,
                                                destination,
                                                location: 1,
                                                zero: true,
                                            }),
                                            ip,
                                            span,
                                        ));
                                        ip += 4;
                                    }
                                }
                            }
                            label => {
                                expanded.push_back((Instruction::Load(label), ip, span));
                                ip += 8;
                            }
                        }
                    }

                    Instruction::Push(StackOp::Registers(regs)) => {
                        for reg in regs {
                            expanded.push_back((
                                Instruction::Push(StackOp::Register(reg)),
                                ip,
                                span.clone(),
                            ));
                            ip += 4;
                        }
                    }

                    Instruction::Pop(StackOp::Registers(regs)) => {
                        for reg in regs {
                            expanded.push_back((
                                Instruction::Pop(StackOp::Register(reg)),
                                ip,
                                span.clone(),
                            ));
                            ip += 4;
                        }
                    }

                    x => {
                        expanded.push_back((x, ip, span));
                        ip += 4;
                    }
                }
            }

            T::Directive(value, span) => {
                use crate::parse::Directive::*;
                match &value {
                    &Location(address) => {
                        if address & 0xFFFF_0000 == STACK_PAGE {
                            return Err(Error::WritingToStack { span });
                        }
                        if address & 0xFFFF_0000 == ZERO_PAGE {
                            return Err(Error::WritingToZeroPage { span });
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
            IntTestOp as ITO, Jump as J, MemoryLoadOp as MLO, MemoryStoreOp as MSO, StackOp as SO,
        };
        use libseis::instruction_set::{
            control::Jump,
            floating_point, integer, register,
            ControlOp::*,
            FloatingPointOp::*,
            Instruction::{Control, FloatingPoint as FP, Integer, Register},
            IntegerOp::*,
            RegisterOp::*,
        };

        if pages.page_of(address).len > address & 0x0000_FFFF {
            println!("At {span}:\nBy writing to address {address:#x}, you may potentially be overwriting code or data. It is recommended that you move the code or data elsewhere.")
        }

        let write = |(b, a)| pages.page_of(a).write(a, b);

        macro_rules! transform {
            (jump $j:ident) => {
                match $j {
                    J::Absolute(reg) => Ok(Jump::Register(reg)),
                    J::Relative(rel) => Ok(Jump::Relative((rel as SWord) << 2)),
                    J::Label(label_name) => {
                        if let Some(label) = labels.get(&label_name) {
                            let laddr = label.address;
                            let dist = (laddr.wrapping_sub(address)) as SWord;
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
                }
            };
            (ibo $b:ident) => {
                match $b {
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
                }
            };
            (ico $c:ident) => {
                match $c {
                    ICO::RegReg {
                        left,
                        right,
                        signed,
                    } => Ok(integer::CompOp::Registers(left, right, signed)),
                    ICO::RegImm {
                        left,
                        right,
                        signed,
                    } => Ok(integer::CompOp::Immediate(left, right, signed)),
                    ICO::RegConst {
                        left,
                        right,
                        signed,
                    } => {
                        if let Some(rvalue) = constants.get(&right) {
                            use crate::parse::ConstantValue as T;
                            let value = match rvalue.value {
                                T::Integer(i) => i,
                                T::Float(_) => {
                                    return Err(Error::IntTypeMismatch {
                                        name: right,
                                        span: span.clone(),
                                    })
                                }
                            };

                            if rvalue.bits > 15 {
                                Err(Error::ConstTooLong {
                                    name: right,
                                    span: span.clone(),
                                })
                            } else {
                                Ok(integer::CompOp::Immediate(left, value, signed))
                            }
                        } else {
                            Err(Error::NonExistingConstant {
                                name: right,
                                usage: span.clone(),
                            })
                        }
                    }
                }
            };
            (ito $c:ident) => {
                match $c {
                    ITO::RegReg { left, right } => Ok(integer::TestOp::Registers(left, right)),
                    ITO::RegImm { left, right } => Ok(integer::TestOp::Immediate(left, right)),
                    ITO::RegConst { left, right } => {
                        if let Some(rvalue) = constants.get(&right) {
                            use crate::parse::ConstantValue as T;
                            let value = match rvalue.value {
                                T::Integer(i) => i,
                                T::Float(_) => {
                                    return Err(Error::IntTypeMismatch {
                                        name: right,
                                        span: span.clone(),
                                    })
                                }
                            };

                            if rvalue.bits > 15 {
                                Err(Error::ConstTooLong {
                                    name: right,
                                    span: span.clone(),
                                })
                            } else {
                                Ok(integer::TestOp::Immediate(left, value))
                            }
                        } else {
                            Err(Error::NonExistingConstant {
                                name: right,
                                usage: span.clone(),
                            })
                        }
                    }
                }
            };
            (fbo $b:ident) => {
                floating_point::BinaryOp($b.source, $b.opt, $b.destination)
            };
            (fuo $u:ident) => {
                floating_point::UnaryOp($u.source, $u.destination)
            };
            (push $p:ident) => {
                match $p {
                    SO::Register(v) => v,
                    _ => unreachable!(),
                }
            };
            (pop $p:ident) => {
                match $p {
                    SO::Register(v) => v,
                    _ => unreachable!(),
                }
            };
            (load $l:ident) => {
                match $l {
                    MLO::Zpg {
                        address,
                        destination,
                    } => Ok(register::ReadOp::ZeroPage {
                        address,
                        destination,
                    }),
                    MLO::ConstZpg {
                        constant,
                        destination,
                    } => {
                        if let Some(cvalue) = constants.get(&constant) {
                            use crate::parse::ConstantValue as T;
                            let value = match cvalue.value {
                                T::Integer(i) => i,
                                T::Float(_) => {
                                    return Err(Error::IntTypeMismatch {
                                        name: constant,
                                        span: span.clone(),
                                    })
                                }
                            };

                            if cvalue.bits > 16 {
                                Err(Error::ConstTooLong {
                                    name: constant,
                                    span: span.clone(),
                                })
                            } else {
                                Ok(register::ReadOp::ZeroPage {
                                    address: value as Short,
                                    destination,
                                })
                            }
                        } else {
                            Err(Error::NonExistingConstant {
                                name: constant,
                                usage: span.clone(),
                            })
                        }
                    }
                    MLO::Indirect {
                        address,
                        destination,
                        volatile,
                    } => Ok(register::ReadOp::Indirect {
                        volatile,
                        address,
                        destination,
                    }),
                    MLO::Offset {
                        address,
                        offset,
                        destination,
                        volatile,
                    } => Ok(register::ReadOp::OffsetIndirect {
                        address,
                        offset,
                        destination,
                        volatile,
                    }),
                    MLO::Indexed {
                        address,
                        index,
                        destination,
                        volatile,
                    } => Ok(register::ReadOp::IndexedIndirect {
                        address,
                        index,
                        destination,
                        volatile,
                    }),
                    MLO::Stack {
                        offset,
                        destination,
                    } => Ok(register::ReadOp::StackOffset {
                        offset,
                        destination,
                    }),
                }
            };
            (stor $s:ident) => {
                match $s {
                    MSO::Zpg { address, source } => {
                        Ok(register::WriteOp::ZeroPage { address, source })
                    }
                    MSO::ConstZpg { constant, source } => {
                        if let Some(cvalue) = constants.get(&constant) {
                            use crate::parse::ConstantValue as T;
                            let value = match cvalue.value {
                                T::Integer(i) => i,
                                T::Float(_) => {
                                    return Err(Error::IntTypeMismatch {
                                        name: constant,
                                        span: span.clone(),
                                    })
                                }
                            };

                            if cvalue.bits > 16 {
                                Err(Error::ConstTooLong {
                                    name: constant,
                                    span: span.clone(),
                                })
                            } else {
                                Ok(register::WriteOp::ZeroPage {
                                    address: value as Short,
                                    source,
                                })
                            }
                        } else {
                            Err(Error::NonExistingConstant {
                                name: constant,
                                usage: span.clone(),
                            })
                        }
                    }
                    MSO::Indirect {
                        address,
                        source,
                        volatile,
                    } => Ok(register::WriteOp::Indirect {
                        volatile,
                        address,
                        source,
                    }),
                    MSO::Offset {
                        address,
                        offset,
                        source,
                        volatile,
                    } => Ok(register::WriteOp::OffsetIndirect {
                        address,
                        offset,
                        source,
                        volatile,
                    }),
                    MSO::Indexed {
                        address,
                        index,
                        source,
                        volatile,
                    } => Ok(register::WriteOp::IndexedIndirect {
                        address,
                        index,
                        source,
                        volatile,
                    }),
                    MSO::Stack { offset, source } => {
                        Ok(register::WriteOp::StackOffset { offset, source })
                    }
                }
            };
        }

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
            I::Jmp(j) => Control(Jmp(transform!(jump j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jsr(j) => Control(Jsr(transform!(jump j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jeq(j) => Control(Jeq(transform!(jump j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jne(j) => Control(Jne(transform!(jump j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jgt(j) => Control(Jgt(transform!(jump j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jlt(j) => Control(Jlt(transform!(jump j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jge(j) => Control(Jge(transform!(jump j)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Jle(j) => Control(Jle(transform!(jump j)?))
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

            I::Add(b) => Integer(Add(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Sub(b) => Integer(Sub(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Mul(b) => Integer(Mul(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Dvu(b) => Integer(Dvu(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Dvs(b) => Integer(Dvs(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Mod(b) => Integer(Mod(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::And(b) => Integer(And(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Ior(b) => Integer(Ior(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Xor(b) => Integer(Xor(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Bsl(b) => Integer(Bsl(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Bsr(b) => Integer(Bsr(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Asr(b) => Integer(Asr(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Rol(b) => Integer(Rol(transform!(ibo b)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Ror(b) => Integer(Ror(transform!(ibo b)?))
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
            I::Sxt(s) => Integer(Sxt(integer::SignExtendOp(s.width, s.register)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Cmp(c) => Integer(Cmp(transform!(ico c)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Tst(c) => Integer(Tst(transform!(ito c)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),

            I::Fadd(b) => FP(Fadd(transform!(fbo b)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Fsub(b) => FP(Fsub(transform!(fbo b)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Fmul(b) => FP(Fmul(transform!(fbo b)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Fdiv(b) => FP(Fdiv(transform!(fbo b)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Fmod(b) => FP(Fmod(transform!(fbo b)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Fcmp(c) => FP(Fcmp(floating_point::CompOp(c.left, c.right)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Fneg(u) => FP(Fneg(transform!(fuo u)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Frec(u) => FP(Frec(transform!(fuo u)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Itof(u) => FP(Itof(transform!(fuo u)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Ftoi(u) => FP(Ftoi(transform!(fuo u)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Fchk(target) => FP(Fchk(floating_point::CheckOp(target)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),

            I::Push(p) => Register(Push(transform!(push p)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Pop(p) => Register(Pop(transform!(pop p)))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Lbr(l) => Register(Lbr(transform!(load l)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Sbr(s) => Register(Sbr(transform!(stor s)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Lsr(l) => Register(Lsr(transform!(load l)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Ssr(s) => Register(Ssr(transform!(stor s)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Llr(l) => Register(Llr(transform!(load l)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Slr(s) => Register(Slr(transform!(stor s)?))
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write),
            I::Tfr(source, destination) => Register(Tfr(register::RegOp {
                source,
                destination,
            }))
            .encode()
            .to_be_bytes()
            .into_iter()
            .zip(address..)
            .for_each(write),
            I::Ldr(l) => {
                use crate::parse::ImmediateLoadOp as L;
                use libseis::instruction_set::register::ImmOp::*;
                match l {
                    L::ZpgAddr {
                        address,
                        destination,
                    } => Register(Ldr(ZeroPageTranslate {
                        address,
                        destination,
                    })),
                    L::ConstZpgAddr {
                        constant,
                        destination,
                    } => {
                        if let Some(cval) = constants.get(&constant) {
                            use crate::parse::ConstantValue as T;
                            let value = match cval.value {
                                T::Integer(i) => i,
                                T::Float(_) => {
                                    return Err(Error::IntTypeMismatch {
                                        name: constant,
                                        span: span.clone(),
                                    })
                                }
                            };

                            if cval.bits > 16 {
                                return Err(Error::ConstTooLong {
                                    name: constant,
                                    span: span.clone(),
                                });
                            } else {
                                Register(Ldr(ZeroPageTranslate {
                                    address: value as Short,
                                    destination,
                                }))
                            }
                        } else {
                            return Err(Error::NonExistingConstant {
                                name: constant,
                                usage: span.clone(),
                            });
                        }
                    }
                    L::Immediate {
                        value,
                        destination,
                        location,
                        zero,
                    } => Register(Ldr(Immediate {
                        zero,
                        shift: location,
                        immediate: value,
                        destination,
                    })),
                }
                .encode()
                .to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(write)
            }

            I::Load(Label { ident, destination }) => {
                if let Some(label) = labels.get(&ident) {
                    use register::ImmOp::Immediate;
                    let left = (label.address & 0xFFFF_0000) >> 16;
                    let right = label.address & 0x0000_FFFF;

                    Register(Ldr(Immediate {
                        zero: true,
                        shift: 0,
                        immediate: right as Short,
                        destination,
                    }))
                    .encode()
                    .to_be_bytes()
                    .into_iter()
                    .chain(
                        Register(Ldr(Immediate {
                            zero: false,
                            shift: 1,
                            immediate: left as Short,
                            destination,
                        }))
                        .encode()
                        .to_be_bytes()
                        .into_iter(),
                    )
                    .zip(address..)
                    .for_each(write)
                } else {
                    return Err(Error::NonExistingLabel {
                        name: ident,
                        usage: span,
                    });
                }
            }

            _ => unreachable!("Non-label LOAD instructions have already been evaluated."),
        }
    }

    for (data, address, span) in data {
        if pages.page_of(address).len > address & 0x0000_FFFF {
            println!("At {span}:\nBy writing to address {address:#x}, you may potentially be overwriting code or data. It is recommended that you move the code or data elsewhere.")
        }

        data.into_iter()
            .zip(address..)
            .for_each(|(b, a)| pages.page_of(a).write(a, b));
    }

    Ok(pages)
}
