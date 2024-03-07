use std::time::Instant;

use libmem::{
    cache::{Cache, MappedLru, Status},
    memory::Memory,
};
use rand::{
    distributions::{uniform::SampleUniform, DistIter, Distribution, Uniform},
    random, SeedableRng,
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

/// Makes sure that the math governing tag/set/offset bits is OK
#[test]
fn bits() {
    let mut failures = 0;
    println!("| Run | Tag | Set | Offset |   Time   | Result |");
    println!("|----:|----:|----:|-------:|----------|--------|");

    for (i, (off, mut set)) in rng_iter(random(), 2, 30)
        .zip(rng_iter(random(), 1, 31))
        .take(32)
        .enumerate()
    {
        if off + set >= 32 {
            set = 32 - off
        }
        let start = Instant::now();
        let cache = MappedLru::new(off, set);
        let end = Instant::now();

        let result = cache.tag_bits() == 32 - (off + set)
            && cache.set_bits() == set
            && cache.off_bits() == off;

        println!(
            "| {run:3} | {tag:3} | {set:3} | {off:6} | {time:6}ms | {result:6} |",
            tag = 32 - (off + set),
            run = i + 1,
            time = (end - start).as_millis(),
            result = if result { "OK" } else { "Fail" }
        );

        if !result {
            failures += 1;
        }
    }

    if failures > 0 {
        panic!(
            "Failed {failures} round{s}",
            s = if failures == 1 { "" } else { "s" }
        )
    }
}

#[test]
fn read_byte_cold() {
    todo!()
}

#[test]
fn read_short_cold() {
    todo!()
}

#[test]
fn read_word_cold() {
    todo!()
}
