use libmem::cache::Associative;
use rand::{
    distributions::{uniform::SampleUniform, DistIter, Distribution, Uniform},
    SeedableRng,
};
use rand_chacha::ChaCha8Rng as Rng;
use std::time::Instant;

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
///
/// Also measures time (which led me to do an on-write allocate policy)
#[test]
fn bits() {
    let mut failures = 0;
    println!("|   Run   | Tag | Set | Offset |      Time      | Result |");
    println!("|--------:|----:|----:|-------:|---------------:|--------|");

    let runs = 64;
    let mut total_time = 0.0;

    for (i, (off, mut set)) in rng_iter(12, 2, 30)
        .zip(rng_iter(16, 1, 31))
        .take(runs)
        .enumerate()
    {
        if off + set >= 32 {
            set = 32 - off
        }
        let start = Instant::now();
        let cache = Associative::new(off, set);
        let end = Instant::now();

        let result = cache.tag_bits() == 32 - (off + set)
            && cache.set_bits() == set
            && cache.off_bits() == off;

        let time = (end - start).as_secs_f32() * 1e6;

        println!(
            "| {run:7} | {tag:3} | {set:3} | {off:6} | {time:>11.2} µs | {result:6} |",
            tag = 32 - (off + set),
            run = i + 1,
            result = if result { "OK" } else { "Fail" }
        );

        total_time += time;

        if !result {
            failures += 1;
        }
    }

    println!(
        "| average |     |     |        | {avg:11.2} µs |        |",
        avg = total_time / runs as f32
    );

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
