use crate::parse::{tokenize, ImmediateLoadOp, Instruction as InstructionToken, LineType};
use libseis::{
    instruction_set::{
        control::ControlOp::Halt,
        integer::{
            BinaryOp::*,
            IntegerOp::{Add, Mul},
        },
        register::{ImmOp::Immediate, RegisterOp::Ldr},
        Instruction::{self, *},
    },
    registers::V,
};
use std::{error::Error, fs::File, io::Write, path::Path};

#[test]
fn basic_asm() -> Result<(), Box<dyn Error>> {
    const BASIC_CODE: &'static str = r#"
main:
    ldr 1, v0
    ldr 2, v1
    ldr 3, v1.1
    mul v1, 2, v3
    add v0, v1, v2
    halt
"#;
    const EXPECTED_CODE: [Instruction; 6] = [
        Register(Ldr(Immediate {
            zero: true,
            shift: 0,
            immediate: 1,
            destination: V[0],
        })),
        Register(Ldr(Immediate {
            zero: true,
            shift: 0,
            immediate: 2,
            destination: V[1],
        })),
        Register(Ldr(Immediate {
            zero: false,
            shift: 1,
            immediate: 3,
            destination: V[1],
        })),
        Integer(Mul(Immediate(V[1], 2, V[3]))),
        Integer(Add(Registers(V[0], V[1], V[2]))),
        Control(Halt),
    ];

    let file = Path::new("basic.asm");
    File::create(file)?.write_all(BASIC_CODE.as_bytes())?;
    let tokens = tokenize(file)?;
    let mut token_iter = tokens.iter();

    let token = token_iter.next().expect("Could not get first token");
    match token {
        LineType::Label(l, _) => assert_eq!(l, "main"),
        _ => panic!("First token must be a label"),
    }
    let token = token_iter.next().expect("Could not get second token");
    assert!(matches!(
        token,
        LineType::Instruction(
            InstructionToken::Ldr(ImmediateLoadOp::Immediate {
                value: 1,
                destination: 0,
                location: 0,
                insert: false,
            }),
            _,
        ),
    ));
    let token = token_iter.next().expect("Could not get third token");
    assert!(matches!(
        token,
        LineType::Instruction(
            InstructionToken::Ldr(ImmediateLoadOp::Immediate {
                value: 2,
                destination: 1,
                location: 0,
                insert: false
            }),
            _
        )
    ));
    // TODO: check the rest of the tokens

    Ok(())
}
