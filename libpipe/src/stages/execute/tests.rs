use super::*;
use libmem::{cache::Associative, memory::Memory, module::SingleLevel};
use libseis::{instruction_set::ControlOp, pages::PAGE_SIZE};
use std::array::from_fn;

fn basic_setup() -> (SingleLevel, Registers, Locks) {
    // Create a memoryspace where every byte is the index modulo 256
    let mut memory = Memory::new(4);
    memory.set_page(
        0x0000_0000,
        &from_fn::<Byte, PAGE_SIZE, _>(|i| (i & 0xFF) as u8),
    );
    memory.set_page(
        0x0001_0000,
        &from_fn::<Byte, PAGE_SIZE, _>(|i| (i & 0xFF) as u8),
    );
    memory.set_page(
        0x0002_0000,
        &from_fn::<Byte, PAGE_SIZE, _>(|i| (i & 0xFF) as u8),
    );
    memory.set_page(
        0x0003_0000,
        &from_fn::<Byte, PAGE_SIZE, _>(|i| (i & 0xFF) as u8),
    );

    (
        SingleLevel::new(
            Box::new(Associative::new(3, 2)),
            Box::new(Associative::new(3, 2)),
            memory,
            10,
            2,
            false,
        ),
        Registers::default(),
        Locks::default(),
    )
}

#[test]
fn clock_basic() {
    let (mut mem, mut reg, mut lock) = basic_setup();
    let mut execute = Execute::default();

    assert!(matches!(
        execute.clock(Clock::Ready(1), &mut reg, &mut lock, &mut mem),
        Clock::Ready(1)
    ));

    assert!(matches!(
        execute,
        Execute {
            state: State::Idle,
            forward: None
        }
    ));

    assert!(matches!(
        execute.forward(Status::Flow(DecodeResult::Forward {
            instruction: Instruction::Control(ControlOp::Nop),
            regvals: RegMap::default(),
            reglocks: RegisterFlags::default()
        })),
        Status::Stall(1)
    ));
}
