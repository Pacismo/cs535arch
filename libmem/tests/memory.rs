use libmem::memory::Memory;
use libseis::types::Byte;

#[test]
fn test_read_uninit() {
    let mem = Memory::new(4);

    for i in 0x0000_0000..0x0004_0000 {
        assert_eq!(mem.read_byte(i), 0);
    }
}

#[test]
fn test_write_uninit() {
    let mut mem = Memory::new(4);

    for i in 0x0000_0000..0x0004_0000 {
        mem.write_byte(i, (i % 0xFF) as Byte);
        assert_eq!(mem.read_byte(i), (i % 0xFF) as Byte);
    }
}

#[test]
fn test_write_init() {
    let mut mem = Memory::new(4);

    mem.write_byte(0x0000_0000, 32);
    assert_eq!(mem.read_byte(0x0000_0000), 32);

    mem.write_byte(0x0000_0000, 64);
    assert_eq!(mem.read_byte(0x0000_0000), 64);

    mem.write_byte(0x0001_0002, 64);
    assert_eq!(mem.read_byte(0x0001_0002), 64);
}

#[test]
fn test_read_short() {
    let mut mem = Memory::new(4);

    mem.write_byte(0x0000_0000, 0x01);
    mem.write_byte(0x0000_0001, 0xF1);

    assert_eq!(mem.read_short(0x0000_0000), 0x01F1);
}

#[test]
fn test_write_short() {
    let mut mem = Memory::new(4);

    mem.write_short(0x0000_0000, 0x01F1);

    assert_eq!(mem.read_byte(0x0000_0000), 0x01);
    assert_eq!(mem.read_byte(0x0000_0001), 0xF1);
}

#[test]
fn test_read_word() {
    let mut mem = Memory::new(4);

    mem.write_byte(0x0000_0000, 0x01);
    mem.write_byte(0x0000_0001, 0xF1);
    mem.write_byte(0x0000_0002, 0xAB);
    mem.write_byte(0x0000_0003, 0x13);

    assert_eq!(mem.read_word(0x0000_0000), 0x01F1_AB13);
}

#[test]
fn test_write_word() {
    let mut mem = Memory::new(4);

    mem.write_word(0x0000_0000, 0x01F1_AB13);

    assert_eq!(mem.read_byte(0x0000_0000), 0x01);
    assert_eq!(mem.read_byte(0x0000_0001), 0xF1);
    assert_eq!(mem.read_byte(0x0000_0002), 0xAB);
    assert_eq!(mem.read_byte(0x0000_0003), 0x13);
}

#[test]
fn test_write_page_boundary() {
    let mut mem = Memory::new(4);

    mem.write_short(0x0000_FFFF, 0x01F1);

    assert_eq!(
        // Assert that only the first two pages are allocated
        format!("{mem:?}"),
        "Memory { pages: [true, true, false, false] }"
    );

    assert_eq!(mem.read_byte(0x0000_FFFF), 0x01);
    assert_eq!(mem.read_byte(0x0001_0000), 0xF1);
}

#[test]
fn test_read_page_boundary() {
    let mut mem = Memory::new(4);

    mem.write_byte(0x0000_FFFF, 0x01);
    mem.write_byte(0x0001_0000, 0xF1);

    assert_eq!(mem.read_short(0x0000_FFFF), 0x01F1);
}
