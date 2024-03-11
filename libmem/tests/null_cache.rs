use libmem::{
    cache::{Cache, NullCache, Status},
    memory::Memory,
};
use rand::{
    distributions::{uniform::SampleUniform, DistIter, Distribution, Uniform},
    SeedableRng,
};
use rand_chacha::ChaChaRng as Rng;

fn rng_closure<T: SampleUniform>(seed: u64, low: T, high: T) -> impl FnMut() -> T {
    let mut rng = Rng::seed_from_u64(seed);
    let dev = Uniform::new_inclusive(low, high);

    move || dev.sample(&mut rng)
}

fn rng_iter<T: SampleUniform>(seed: u64, low: T, high: T) -> DistIter<Uniform<T>, Rng, T> {
    let rng = Rng::seed_from_u64(seed);
    let dev = Uniform::new_inclusive(low, high);

    dev.sample_iter(rng)
}

#[test]
fn get_byte_miss() {
    let mut cache = NullCache::new();

    for address in rng_iter(0, 0x0000_0000, 0xFFFF_FFFF).take(32) {
        assert!(matches!(cache.get_byte(address), Err(Status::Disabled)))
    }
}

#[test]
fn get_short_miss() {
    let mut cache = NullCache::new();

    for address in rng_iter(0, 0x0000_0000, 0xFFFF_FFFE).take(32) {
        assert!(matches!(cache.get_short(address), Err(Status::Disabled)))
    }
}

#[test]
fn get_word_miss() {
    let mut cache = NullCache::new();

    for address in rng_iter(0, 0x0000_0000, 0xFFFF_FFFC).take(32) {
        assert!(matches!(cache.get_word(address), Err(Status::Disabled)))
    }
}

#[test]
fn write_byte_miss() {
    let mut cache = NullCache::new();
    let mut gen = rng_closure(32, 0, 255);

    for address in rng_iter(0, 0x0000_0000, 0xFFFF_FFFF).take(32) {
        assert!(cache.write_byte(address, gen()).is_miss());
    }
}

#[test]
fn write_short_miss() {
    let mut cache = NullCache::new();
    let mut gen = rng_closure(32, 0, 65535);

    for address in rng_iter(0, 0x0000_0000, 0xFFFF_FFFE).take(32) {
        assert!(cache.write_short(address, gen()).is_miss());
    }
}

#[test]
fn write_word_miss() {
    let mut cache = NullCache::new();
    let mut gen = rng_closure(32, 0, 4294967295);

    for address in rng_iter(0, 0x0000_0000, 0xFFFF_FFFC).take(32) {
        assert!(cache.write_word(address, gen()).is_miss());
    }
}

#[test]
fn read_line() {
    let mut cache = NullCache::new();
    let mut memory = Memory::new(4);
    let mut gen = rng_closure(32, 0, 4096);

    let val = gen();
    memory.write_word(0x0000_0000, val);

    assert!(!cache.write_line(0x0000_0000, &mut memory).disabled());

    assert!(matches!(cache.get_word(0x0000_0000), Err(Status::Disabled)))
}

#[test]
fn line_len() {
    assert!(NullCache::new().line_len() == 0)
}
