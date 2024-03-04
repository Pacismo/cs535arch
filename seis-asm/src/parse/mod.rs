mod asm_parser;
mod error;
mod lines;

use self::{
    asm_parser::Rule,
    error::{Error, ErrorSource},
    lines::{Constant, Directive, Instruction, Jump, LineType, Lines, Span},
};
use asm_parser::AsmParser;
use libseis::registers;
use pest::{
    error::{Error as PestError, ErrorVariant},
    iterators::{Pair, Pairs},
    Parser,
};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

/// Expands to a [`convert_base`], followed by a [`std::ops::FromResidual`], a parse and an unwrap.
macro_rules! parse_integer {
    ($i:expr) => {
        convert_base($i)?.parse::<i64>().unwrap() as _
    };
}

/// The value passed must be any one of [`Rule::oct`], [`Rule::dec`], or [`Rule::hex`].
///
/// Converts the integer into a base-10 representation.
fn convert_base(pair: Pair<'_, Rule>) -> Result<String, ErrorSource> {
    let str = pair.as_str();
    let sign = if str.starts_with('-') {
        Some(true)
    } else if str.starts_with('+') {
        Some(false)
    } else {
        None
    };
    let str = if sign.is_some() { &str[1..] } else { str };

    let value = match pair.as_rule() {
        Rule::hex => u64::from_str_radix(&str[2..], 16),
        Rule::dec => u64::from_str_radix(str, 10),
        Rule::oct => u64::from_str_radix(&str[1..], 8),
        _ => unreachable!("{}", pair.as_span().as_str()),
    }
    .map_err(|e| {
        PestError::new_from_span(
            ErrorVariant::CustomError {
                message: format!("Couldn't parse integer: {e}"),
            },
            pair.as_span(),
        )
    })?;

    if let Some(true) = sign {
        Ok(format!("-{}", value))
    } else {
        Ok(format!("{}", value))
    }
}

fn tokenize_constant(mut pair: Pairs<'_, Rule>) -> Result<Constant, ErrorSource> {
    use lines::ConstantValue::*;

    let name = pair.next().unwrap().into_inner().next().unwrap().as_str();
    let value = match pair.next().unwrap() {
        s if s.as_rule() == Rule::integer => match pair.next().unwrap().as_rule() {
            Rule::byte => Byte(s.as_str().to_owned()),
            Rule::short => Short(s.as_str().to_owned()),
            Rule::word => Word(s.as_str().to_owned()),
            _ => unreachable!(),
        },
        s if s.as_rule() == Rule::float => Float(s.as_str().to_owned()),
        s if s.as_rule() == Rule::char => Byte(s.as_str()[1..2].to_owned()),
        s if s.as_rule() == Rule::string => {
            let string = s.as_str();
            String(string[1..string.len() - 1].to_owned())
        }

        _ => unreachable!(),
    };

    Ok(Constant {
        ident: name.to_owned(),
        value,
    })
}

fn tokenize_directive(mut pair: Pairs<'_, Rule>) -> Result<Directive, ErrorSource> {
    let ident = pair.next().unwrap();
    let value = pair.next();

    match ident.as_str().to_lowercase().as_str() {
        // TODO: add the ZeroPage directive
        "location" => {
            let value = value.ok_or_else(|| {
                PestError::new_from_pos(
                    ErrorVariant::CustomError {
                        message: "\"location\" expects an address".to_owned(),
                    },
                    ident.as_span().end_pos(),
                )
            })?;

            Ok(Directive::Location(parse_integer!(value
                .into_inner()
                .next()
                .unwrap())))
        }
        "public" => {
            if value.is_none() {
                Ok(Directive::Public)
            } else {
                Err(PestError::new_from_span(
                    ErrorVariant::CustomError {
                        message: "\"public\" does not expect a value".into(),
                    },
                    value.unwrap().as_span(),
                )
                .into())
            }
        }
        x => Err(PestError::new_from_span(
            ErrorVariant::CustomError {
                message: format!("Did not recognize directive \"{x}\""),
            },
            ident.as_span(),
        )
        .into()),
    }
}

fn tokenize_instruction(mut pair: Pairs<'_, Rule>) -> Result<Instruction, ErrorSource> {
    let instruction = pair.next().unwrap();

    match instruction.as_rule() {
        Rule::halt => Ok(Instruction::Halt),
        Rule::nop => Ok(Instruction::Nop),
        Rule::ret => Ok(Instruction::Ret),
        // The method of decoding a Jump is the same; simply determine the rule later.
        x @ (Rule::jmp
        | Rule::jsr
        | Rule::jeq
        | Rule::jne
        | Rule::jgt
        | Rule::jlt
        | Rule::jge
        | Rule::jle) => {
            use Instruction::{Jeq, Jge, Jgt, Jle, Jlt, Jmp, Jne, Jsr};

            let inner = instruction.into_inner().next().unwrap();
            let mode = match inner.as_rule() {
                Rule::ident => Jump::Label(inner.as_str().to_owned()),
                Rule::relative => Jump::Relative(parse_integer!(inner
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap())),
                Rule::absolute => Jump::Absolute(registers::get_id(inner.as_str()).unwrap()),
                _ => unreachable!(),
            };

            match x {
                Rule::jmp => Ok(Jmp(mode)),
                Rule::jsr => Ok(Jsr(mode)),
                Rule::jeq => Ok(Jeq(mode)),
                Rule::jne => Ok(Jne(mode)),
                Rule::jgt => Ok(Jgt(mode)),
                Rule::jlt => Ok(Jlt(mode)),
                Rule::jge => Ok(Jge(mode)),
                Rule::jle => Ok(Jle(mode)),
                _ => unreachable!("Other variants can never be present in this branch"),
            }
        }

        // Similarly to jumping, binary operations share the same parsing, but the rule may be determined later
        x @ (Rule::add
        | Rule::sub
        | Rule::mul
        | Rule::dvu
        | Rule::dvs
        | Rule::r#mod
        | Rule::and
        | Rule::ior
        | Rule::xor
        | Rule::bsl
        | Rule::bsr
        | Rule::asr
        | Rule::rol
        | Rule::ror) => {
            use lines::IntBinaryOp::*;
            use Instruction::{
                Add, And, Asr, Bsl, Bsr, Dvs, Dvu, Ior, Mod, Mul, Rol, Ror, Sub, Xor,
            };

            let mut inner = instruction.into_inner();
            let source = registers::get_id(inner.next().unwrap().as_str()).unwrap();
            let opt = inner.next().unwrap();
            inner.next();
            let destination = registers::get_id(inner.next().unwrap().as_str()).unwrap();

            let mode = match opt.as_rule() {
                Rule::vareg => RegReg {
                    source,
                    opt: registers::get_id(opt.as_str()).unwrap(),
                    destination,
                },
                Rule::integer => RegImm {
                    source,
                    opt: parse_integer!(opt.into_inner().next().unwrap()),
                    destination,
                },
                Rule::ident => RegConst {
                    source,
                    opt: opt.as_str().to_owned(),
                    destination,
                },
                _ => unreachable!(),
            };

            Ok(match x {
                Rule::add => Add(mode),
                Rule::sub => Sub(mode),
                Rule::mul => Mul(mode),
                Rule::dvu => Dvu(mode),
                Rule::dvs => Dvs(mode),
                Rule::r#mod => Mod(mode),
                Rule::and => And(mode),
                Rule::ior => Ior(mode),
                Rule::xor => Xor(mode),
                Rule::bsl => Bsl(mode),
                Rule::bsr => Bsr(mode),
                Rule::asr => Asr(mode),
                Rule::rol => Rol(mode),
                Rule::ror => Ror(mode),
                _ => unreachable!("Other variants can never be in this branch"),
            })
        }
        Rule::sxt => {
            use lines::IntSignExtendOp;
            use Instruction::Sxt;

            let mut inner = instruction.into_inner();
            let width = match inner.next().unwrap().as_str() {
                "byte" => 0,
                "short" => 1,
                "word" => 2,
                _ => unreachable!(),
            };
            let register = registers::get_id(inner.next().unwrap().as_str()).unwrap();

            Ok(Sxt(IntSignExtendOp { register, width }))
        }
        Rule::not => {
            use lines::IntUnaryOp;
            use Instruction::Not;

            let mut inner = instruction.into_inner();
            let source = registers::get_id(inner.next().unwrap().as_str()).unwrap();
            inner.next();
            let destination = registers::get_id(inner.next().unwrap().as_str()).unwrap();

            Ok(Not(IntUnaryOp {
                source,
                destination,
            }))
        }
        x @ (Rule::cmp | Rule::tst) => {
            use lines::IntCompOp::*;
            use Instruction::{Cmp, Tst};

            let mut inner = instruction.into_inner();
            let left = registers::get_id(inner.next().unwrap().as_str()).unwrap();
            let right = inner.next().unwrap();

            let mode = match right.as_rule() {
                Rule::integer => RegImm {
                    left,
                    right: parse_integer!(right.into_inner().next().unwrap()),
                },
                Rule::vareg => RegReg {
                    left,
                    right: registers::get_id(right.as_str()).unwrap(),
                },
                Rule::ident => RegConst {
                    left,
                    right: right.as_str().to_owned(),
                },
                _ => unreachable!(),
            };

            Ok(match x {
                Rule::cmp => Cmp(mode),
                Rule::tst => Tst(mode),
                _ => unreachable!(),
            })
        }

        x @ (Rule::fadd | Rule::fsub | Rule::fmul | Rule::fdiv | Rule::fmod) => {
            use lines::FloatBinaryOp;
            use Instruction::{Fadd, Fdiv, Fmod, Fmul, Fsub};

            let mut inner = instruction.into_inner();
            let source = registers::get_id(inner.next().unwrap().as_str()).unwrap();
            let opt = registers::get_id(inner.next().unwrap().as_str()).unwrap();
            inner.next();
            let destination = registers::get_id(inner.next().unwrap().as_str()).unwrap();

            Ok(match x {
                Rule::fadd => Fadd(FloatBinaryOp {
                    source,
                    opt,
                    destination,
                }),
                Rule::fsub => Fsub(FloatBinaryOp {
                    source,
                    opt,
                    destination,
                }),
                Rule::fmul => Fmul(FloatBinaryOp {
                    source,
                    opt,
                    destination,
                }),
                Rule::fdiv => Fdiv(FloatBinaryOp {
                    source,
                    opt,
                    destination,
                }),
                Rule::fmod => Fmod(FloatBinaryOp {
                    source,
                    opt,
                    destination,
                }),
                _ => unreachable!(),
            })
        }
        Rule::fcmp => {
            use lines::FloatCompOp;
            use Instruction::Fcmp;

            let mut inner = instruction.into_inner();
            let left = registers::get_id(inner.next().unwrap().as_str()).unwrap();
            let right = registers::get_id(inner.next().unwrap().as_str()).unwrap();

            Ok(Fcmp(FloatCompOp { left, right }))
        }
        x @ (Rule::fneg | Rule::frec | Rule::itof | Rule::ftoi) => {
            use lines::FloatUnaryOp;
            use Instruction::{Fneg, Frec, Ftoi, Itof};

            let mut inner = instruction.into_inner();
            let source = registers::get_id(inner.next().unwrap().as_str()).unwrap();
            inner.next();
            let destination = registers::get_id(inner.next().unwrap().as_str()).unwrap();

            Ok(match x {
                Rule::fneg => Fneg(FloatUnaryOp {
                    source,
                    destination,
                }),
                Rule::frec => Frec(FloatUnaryOp {
                    source,
                    destination,
                }),
                Rule::itof => Itof(FloatUnaryOp {
                    source,
                    destination,
                }),
                Rule::ftoi => Ftoi(FloatUnaryOp {
                    source,
                    destination,
                }),
                _ => unreachable!(),
            })
        }
        Rule::fchk => {
            use Instruction::Fchk;

            let register =
                registers::get_id(instruction.into_inner().next().unwrap().as_str()).unwrap();

            Ok(Fchk(register))
        }

        x @ (Rule::push | Rule::pop) => {
            use lines::StackOp::*;
            use Instruction::{Pop, Push};

            let mode = instruction.into_inner().next().unwrap();

            let mode = match mode.as_rule() {
                Rule::regstack => {
                    let mut regs = vec![];

                    for reg in mode.into_inner() {
                        let regid = registers::get_id(reg.as_str()).unwrap();

                        // Registers cannot appear more than once at a time
                        if regs.contains(&regid) {
                            return Err(PestError::new_from_span(
                                ErrorVariant::CustomError {
                                    message: format!(
                                        "Register {} cannot appear more than once",
                                        reg.as_str()
                                    ),
                                },
                                reg.as_span(),
                            )
                            .into());
                        }

                        regs.push(regid);
                    }

                    Registers(regs)
                }
                Rule::oct | Rule::dec | Rule::hex => ExtendOrShrink(parse_integer!(mode)),
                _ => unreachable!(),
            };

            Ok(match x {
                Rule::push => Push(mode),
                Rule::pop => Pop(mode),
                _ => unreachable!(),
            })
        }
        x @ (Rule::lbr | Rule::lsr | Rule::llr) => {
            use lines::MemoryLoadOp::*;
            use Instruction::{Lbr, Llr, Lsr};

            let mut inner = instruction.into_inner();
            let mode = inner.next().unwrap();
            inner.next();
            let destination = registers::get_id(inner.next().unwrap().as_str()).unwrap();

            let mode = match mode.as_rule() {
                Rule::zpgaddr => {
                    let address = parse_integer!(mode.into_inner().next().unwrap());

                    Zpg {
                        address,
                        destination,
                    }
                }
                Rule::offsetind => {
                    let mut inner = mode.into_inner();
                    let address = registers::get_id(inner.next().unwrap().as_str()).unwrap();
                    let offset = parse_integer!(inner.next().unwrap());

                    Offset {
                        address,
                        offset,
                        destination,
                    }
                }
                Rule::indexind => {
                    let mut inner = mode.into_inner();
                    let address = registers::get_id(inner.next().unwrap().as_str()).unwrap();
                    let index = registers::get_id(inner.next().unwrap().as_str()).unwrap();

                    Indexed {
                        address,
                        index,
                        destination,
                    }
                }
                Rule::vareg => {
                    let address = registers::get_id(mode.as_str()).unwrap();

                    Indirect {
                        address,
                        destination,
                    }
                }
                Rule::stackoff => {
                    let offset = parse_integer!(mode.into_inner().next().unwrap());

                    Stack {
                        offset,
                        destination,
                    }
                }
                _ => unreachable!(),
            };

            Ok(match x {
                Rule::lbr => Lbr(mode),
                Rule::lsr => Lsr(mode),
                Rule::llr => Llr(mode),
                _ => unreachable!(),
            })
        }
        x @ (Rule::sbr | Rule::ssr | Rule::slr) => {
            use lines::MemoryLoadOp::*;
            use Instruction::{Sbr, Slr, Ssr};

            let mut inner = instruction.into_inner();
            let destination = registers::get_id(inner.next().unwrap().as_str()).unwrap();
            inner.next();
            let mode = inner.next().unwrap();

            let mode = match mode.as_rule() {
                Rule::zpgaddr => {
                    let address = parse_integer!(mode.into_inner().next().unwrap());

                    Zpg {
                        address,
                        destination,
                    }
                }
                Rule::offsetind => {
                    let mut inner = mode.into_inner();
                    let address = registers::get_id(inner.next().unwrap().as_str()).unwrap();
                    let offset = parse_integer!(inner.next().unwrap());

                    Offset {
                        address,
                        offset,
                        destination,
                    }
                }
                Rule::indexind => {
                    let mut inner = mode.into_inner();
                    let address = registers::get_id(inner.next().unwrap().as_str()).unwrap();
                    let index = registers::get_id(inner.next().unwrap().as_str()).unwrap();

                    Indexed {
                        address,
                        index,
                        destination,
                    }
                }
                Rule::vareg => {
                    let address = registers::get_id(mode.as_str()).unwrap();

                    Indirect {
                        address,
                        destination,
                    }
                }
                Rule::stackoff => {
                    let offset = parse_integer!(mode.into_inner().next().unwrap());

                    Stack {
                        offset,
                        destination,
                    }
                }
                _ => unreachable!(),
            };

            Ok(match x {
                Rule::sbr => Sbr(mode),
                Rule::ssr => Ssr(mode),
                Rule::slr => Slr(mode),
                _ => unreachable!(),
            })
        }
        Rule::tfr => {
            use Instruction::Tfr;

            let mut inner = instruction.into_inner();
            let source = registers::get_id(inner.next().unwrap().as_str()).unwrap();
            let destination = registers::get_id(inner.next().unwrap().as_str()).unwrap();

            Ok(Tfr(source, destination))
        }
        Rule::ldr => {
            use lines::ImmediateLoadOp::*;
            use Instruction::Ldr;

            let mut inner = instruction.into_inner();
            let opt = inner.next().unwrap();
            inner.next();
            let destination = registers::get_id(inner.next().unwrap().as_str()).unwrap();
            let part = inner.next();

            match opt.as_rule() {
                Rule::immload => {
                    if let Some(part) = part {
                        Ok(Ldr(Immediate {
                            value: parse_integer!(opt.into_inner().next().unwrap()),
                            destination,
                            location: part.as_str().parse().unwrap(),
                            insert: true,
                        }))
                    } else {
                        Ok(Ldr(Immediate {
                            value: parse_integer!(opt.into_inner().next().unwrap()),
                            destination,
                            location: 0,
                            insert: false,
                        }))
                    }
                }
                Rule::zpaload => {
                    if let Some(part) = part {
                        return Err(PestError::new_from_span(ErrorVariant::CustomError { message: format!("A zero-page address reference cannot be loaded as a partial value") }, part.as_span()).into());
                    }

                    Ok(Ldr(ZpgAddr {
                        address: parse_integer!(opt.into_inner().next().unwrap()),
                        destination,
                    }))
                }
                _ => unreachable!(),
            }
        }
        Rule::load => {
            use lines::ExpandableLoadOp::*;
            use Instruction::Load;

            let mut inner = instruction.into_inner();

            let opt = inner.next().unwrap();
            inner.next();
            let destination = registers::get_id(inner.next().unwrap().as_str()).unwrap();

            Ok(Load(match opt.as_rule() {
                Rule::float => Float {
                    value: opt.as_str().parse().unwrap(),
                    destination,
                },
                Rule::integer => Integer {
                    value: parse_integer!(opt.into_inner().next().unwrap()),
                    destination,
                },
                Rule::label => Label {
                    ident: opt.into_inner().next().unwrap().as_str().to_owned(),
                    destination,
                },
                Rule::constant => ConstantVal {
                    ident: opt.into_inner().next().unwrap().as_str().to_owned(),
                    destination,
                },
                Rule::constref => ConstantRef {
                    ident: opt.into_inner().next().unwrap().as_str().to_owned(),
                    destination,
                },
                _ => unreachable!(),
            }))
        }

        _ => unreachable!(),
    }
}

fn tokenize_line(line: Pair<'_, Rule>, span: Span) -> Result<Option<LineType>, ErrorSource> {
    use LineType::*;

    Ok(match line.as_rule() {
        Rule::constant => Some(Constant(tokenize_constant(line.into_inner())?, span)),
        Rule::instruction => Some(Instruction(tokenize_instruction(line.into_inner())?, span)),
        Rule::directive => Some(Directive(tokenize_directive(line.into_inner())?, span)),
        Rule::label => Some(Label(line.into_inner().next().unwrap().as_str().to_owned())),

        Rule::EOI => None,
        _ => unreachable!("{line:#?}"),
    })
}

/// First pass: tokenize the input.
///
/// This makes it easier to parse the data.
pub fn tokenize(path: &Path) -> Result<Lines, Error> {
    // TODO: parse using parser
    let mut file = BufReader::new(File::open(path).map_err(|e| Error::new(path, e.into()))?);
    let mut content = String::new();

    file.read_to_string(&mut content)
        .map_err(|e| Error::new(path, e.into()))?;
    let parsed =
        AsmParser::parse(Rule::program, &content).map_err(|e| Error::new(path, e.into()))?;

    let mut lines = Lines::new();

    for (line, number) in parsed.zip(1..) {
        match tokenize_line(line, Span::new(path, number)) {
            Ok(Some(result)) => lines.push_back(result),
            Ok(None) => break,

            Err(e) => return Err(Error::new(path, e)),
        }
    }

    Ok(lines)
}
