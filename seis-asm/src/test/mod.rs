use crate::{
    linker::link_symbols,
    parse::{tokenize, ImmediateLoadOp, Instruction as InstructionToken, IntBinaryOp, LineType},
};
use libseis::{
    instruction_set::{
        control::ControlOp::Halt,
        encode,
        integer::{
            BinaryOp::*,
            IntegerOp::{Add, Mul},
        },
        register::{ImmOp::Immediate, RegisterOp::Ldr},
        Instruction::{self, *},
    },
    registers::V,
    types::Word,
};
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    path::Path,
};

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

    /// Represented visually to help match against above code.
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
    assert!(
        matches!(
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
        ),
        "{token:#?}"
    );
    let token = token_iter.next().expect("Could not get third token");
    assert!(
        matches!(
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
        ),
        "{token:#?}"
    );
    let token = token_iter.next().expect("Could not get fourth token");
    assert!(
        matches!(
            token,
            LineType::Instruction(
                InstructionToken::Ldr(ImmediateLoadOp::Immediate {
                    value: 3,
                    destination: 1,
                    location: 1,
                    insert: true
                }),
                _
            )
        ),
        "{token:#?}"
    );
    let token = token_iter.next().expect("Could not get fifth token");
    assert!(
        matches!(
            token,
            LineType::Instruction(
                InstructionToken::Mul(IntBinaryOp::RegImm {
                    source: 1,
                    opt: 2,
                    destination: 3
                }),
                _
            )
        ),
        "{token:#?}"
    );
    let token = token_iter.next().expect("Could not get sixth token");
    assert!(
        matches!(
            token,
            LineType::Instruction(
                InstructionToken::Add(IntBinaryOp::RegReg {
                    source: 0,
                    opt: 1,
                    destination: 2
                }),
                _
            )
        ),
        "{token:#?}"
    );
    let token = token_iter.next().expect("Could not get seventh token");
    assert!(
        matches!(token, LineType::Instruction(InstructionToken::Halt, _)),
        "{token:#?}"
    );
    assert!(token_iter.next().is_none(), "There should be seven tokens");
    drop(token_iter);

    let result = link_symbols(tokens).expect("Should've linked properly");

    result.write(File::create(file)?)?;
    let mut reader = File::open(file)?;
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes)?;

    for (word, expected) in bytes.chunks_exact(4).zip(EXPECTED_CODE.map(encode)) {
        let word = Word::from_be_bytes([word[0], word[1], word[2], word[3]]);

        // Encoded expected should match fetched word.
        assert_eq!(word, expected);
    }

    // Delete the file -- it is useless.
    std::fs::remove_file(file).expect(&format!("Could not delete file {}", file.display()));

    Ok(())
}
