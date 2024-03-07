use libmem::memory::Memory;
use libseis::types::Byte;

#[test]
fn read_uninit() {
    let mem = Memory::new(2);

    for i in 0x0000_0000..0x0002_0000 {
        assert_eq!(mem.read_byte(i), 0);
    }
}

#[test]
fn write_uninit() {
    let mut mem = Memory::new(2);

    for i in 0x0000_0000..0x0002_0000 {
        mem.write_byte(i, (i % 0xFF) as Byte);
        assert_eq!(mem.read_byte(i), (i % 0xFF) as Byte);
    }
}

#[test]
fn write_init() {
    let mut mem = Memory::new(2);

    mem.write_byte(0x0000_0000, 32);
    assert_eq!(mem.read_byte(0x0000_0000), 32);

    mem.write_byte(0x0000_0000, 64);
    assert_eq!(mem.read_byte(0x0000_0000), 64);

    mem.write_byte(0x0001_0002, 64);
    assert_eq!(mem.read_byte(0x0001_0002), 64);
}

#[test]
fn read_short() {
    let mut mem = Memory::new(2);

    mem.write_byte(0x0000_0000, 0x01);
    mem.write_byte(0x0000_0001, 0xF1);
    mem.write_byte(0x0000_0002, 0xAB);
    mem.write_byte(0x0000_0003, 0xCD);

    assert_eq!(mem.read_short(0x0000_0000), 0x01F1, "Same word");
    assert_eq!(mem.read_short(0x0000_0001), 0xF1AB, "Same word, misaligned");
    assert_eq!(mem.read_short(0x0000_0002), 0xABCD, "Same word, upper-end");
}

#[test]
fn read_short_boundary() {
    let mut mem = Memory::new(2);

    mem.write_byte(0x0000_0003, 0xCD);
    mem.write_byte(0x0000_0004, 0x43);
    mem.write_byte(0x0000_FFFF, 0x01);
    mem.write_byte(0x0001_0000, 0x10);

    assert_eq!(mem.read_short(0x0000_0003), 0xCD43, "Two words");
    assert_eq!(mem.read_short(0x0000_FFFF), 0x0110, "Two pages");
}

#[test]
fn write_short() {
    let mut mem = Memory::new(2);

    mem.write_short(0x0000_0000, 0x01F1);

    assert_eq!(mem.read_byte(0x0000_0000), 0x01);
    assert_eq!(mem.read_byte(0x0000_0001), 0xF1);
}

#[test]
fn write_short_boundary() {
    let mut mem = Memory::new(2);

    mem.write_short(0x0000_0003, 0x01F1);

    assert_eq!(mem.read_byte(0x0000_0003), 0x01, "First byte, same page");
    assert_eq!(mem.read_byte(0x0000_0004), 0xF1, "Second byte, same page");

    mem.write_short(0x0000_FFFF, 0x0FADE);

    assert_eq!(mem.read_byte(0x0000_FFFF), 0xFA, "First byte, first page");
    assert_eq!(mem.read_byte(0x0001_0000), 0xDE, "Second byte, second page");
}

#[test]
fn read_word() {
    let mut mem = Memory::new(2);

    mem.write_byte(0x0000_0000, 0x01);
    mem.write_byte(0x0000_0001, 0xF1);
    mem.write_byte(0x0000_0002, 0xAB);
    mem.write_byte(0x0000_0003, 0x13);

    assert_eq!(mem.read_word(0x0000_0000), 0x01F1_AB13);
}

#[test]
fn read_word_boundary() {
    let mut mem = Memory::new(2);

    mem.write_byte(0x0000_0001, 0x01);
    mem.write_byte(0x0000_0002, 0x02);
    mem.write_byte(0x0000_0003, 0x03);
    mem.write_byte(0x0000_0004, 0x04);
    mem.write_byte(0x0000_0005, 0x05);
    mem.write_byte(0x0000_0006, 0x06);

    assert_eq!(mem.read_word(0x0000_0001), 0x0102_0304, "3:1 word");
    assert_eq!(mem.read_word(0x0000_0002), 0x0203_0405, "2:2 word");
    assert_eq!(mem.read_word(0x0000_0003), 0x0304_0506, "1:3 word");

    mem.write_byte(0x0000_FFFD, 0xFD);
    mem.write_byte(0x0000_FFFE, 0xFE);
    mem.write_byte(0x0000_FFFF, 0xFF);
    mem.write_byte(0x0001_0000, 0x10);
    mem.write_byte(0x0001_0001, 0x11);
    mem.write_byte(0x0001_0002, 0x12);

    assert_eq!(mem.read_word(0x0000_FFFD), 0xFDFE_FF10, "3:1 page");
    assert_eq!(mem.read_word(0x0000_FFFE), 0xFEFF_1011, "2:2 page");
    assert_eq!(mem.read_word(0x0000_FFFF), 0xFF10_1112, "1:3 page");
}

#[test]
fn write_word() {
    let mut mem = Memory::new(2);

    mem.write_word(0x0000_0000, 0x01F1_AB13);

    assert_eq!(mem.read_byte(0x0000_0000), 0x01, "Byte 0 of the word");
    assert_eq!(mem.read_byte(0x0000_0001), 0xF1, "Byte 1 of the word");
    assert_eq!(mem.read_byte(0x0000_0002), 0xAB, "Byte 2 of the word");
    assert_eq!(mem.read_byte(0x0000_0003), 0x13, "Byte 3 of the word");
}

#[test]
fn write_word_boundary() {
    let mut mem = Memory::new(2);

    for offset in 0x0000_0001..0x0000_0004 {
        mem.write_word(offset, 0x0102_0304);

        assert_eq!(mem.read_byte(offset), 0x01, "Byte 0 of misaligned word");
        assert_eq!(mem.read_byte(offset + 1), 0x02, "Byte 1 of misaligned word");
        assert_eq!(mem.read_byte(offset + 2), 0x03, "Byte 2 of misaligned word");
        assert_eq!(mem.read_byte(offset + 3), 0x04, "Byte 3 of misaligned word");
    }

    for offset in 0x0000_FFFD..0x0001_0000 {
        mem.write_word(offset, 0x0102_0304);

        assert_eq!(mem.read_byte(offset), 0x01, "Byte 0 of page-crossing word");
        assert_eq!(
            mem.read_byte(offset + 1),
            0x02,
            "Byte 1 of page-crossing word"
        );
        assert_eq!(
            mem.read_byte(offset + 2),
            0x03,
            "Byte 2 of page-crossing word"
        );
        assert_eq!(
            mem.read_byte(offset + 3),
            0x04,
            "Byte 3 of page-crossing word"
        );
    }
}
