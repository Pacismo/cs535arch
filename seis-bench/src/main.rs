mod cli;
mod config;
mod results;

use crate::cli::Cli;
use clap::Parser;
use config::{Benchmark, SimulationConfig};
use libmem::memory::Memory;
use libpipe::ClockResult;
use libseis::{pages::PAGE_SIZE, types::Word};
use results::RunResult;
use std::{error::Error, fs::File, io::Write, time::Instant};

const PAGES: usize = 16;

pub fn build_binary(benchmark: &Benchmark) -> Result<(), Box<dyn Error>> {
    use std::process::Command;

    Command::new("seis-asm")
        .args(&benchmark.sources)
        .arg("-o")
        .arg(&benchmark.binary)
        .current_dir(&benchmark.path)
        .spawn()?
        .wait()?;

    Ok(())
}

pub fn prepare_sim(mem: &mut Memory, benchmark: &Benchmark) -> Result<(), Box<dyn Error>> {
    use std::fs::read;
    let path = benchmark.path.join(&benchmark.binary);
    let data = read(&path)?;

    if data.len() >= PAGES << 16 {
        return Err(format!("File too long: {}", path.display()).into());
    }

    for (page, data) in data.chunks(PAGE_SIZE).enumerate() {
        mem.set_page((page << 16) as Word, data)
    }

    Ok(())
}

fn run_benchmark<'a>(
    benchmark: &'a Benchmark,
    config: &'a SimulationConfig,
    pipeline_enable: bool,
    cache_enable: bool,
) -> Result<RunResult<'a>, Box<dyn Error>> {
    let mut pipeline = config.build_config(pipeline_enable, cache_enable);
    prepare_sim(pipeline.memory_module_mut().memory_mut(), benchmark)?;

    let mut clocks = 0;
    let mut clocks_required = 1;
    let mut finished = false;

    let start = Instant::now();
    while !finished {
        clocks += clocks_required;
        match pipeline.clock(clocks_required) {
            ClockResult::Stall(clocks) => {
                clocks_required = clocks;
            }
            ClockResult::Flow => {
                clocks_required = 1;
            }
            ClockResult::Dry => {
                finished = true;
            }
        }
    }
    let end = Instant::now();

    Ok(RunResult {
        bench_name: &benchmark.name,
        cache_enable,
        pipeline_enable,
        clocks,
        rtc: end - start,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut config = config::read_configuration(&cli.bench_conf)?;

    println!("Building benchmarks...");

    let conf_path = cli.bench_conf.parent().unwrap();
    config.benchmark.iter_mut().try_for_each(|b| {
        b.path = conf_path.join(&b.path);
        build_binary(b)
    })?;

    println!("Done.");

    let mut results = vec![];
    results.reserve(config.benchmark.len() * 4);

    for (benchmark, pipeline, cache) in config.benchmark.iter().flat_map(|bench| {
        [
            (bench, false, false),
            (bench, false, true),
            (bench, true, false),
            (bench, true, true),
        ]
    }) {
        println!(
            "Doing benchmark \"{}\" with {} and {}...",
            benchmark.name,
            if pipeline {
                "pipelining"
            } else {
                "no pipelining"
            },
            if cache { "cache" } else { "no cache" }
        );

        let run = run_benchmark(benchmark, &config.configuration, pipeline, cache)?;

        println!(
            "Finished benchmark \"{}\" with {} and {}; took {:.2} seconds",
            benchmark.name,
            if pipeline {
                "pipelining"
            } else {
                "no pipelining"
            },
            if cache { "cache" } else { "no cache" },
            run.rtc.as_secs_f64()
        );

        results.push(run);
    }

    let file = cli.output_file();
    println!(
        "Finished benchmarking; writing results to {}",
        file.display()
    );

    let mut file = File::open(file)?;

    writeln!(file, "name, pipeline, cache, clocks, rtc")?;
    results
        .into_iter()
        .try_for_each(|line| writeln!(file, "{}", line))?;

    Ok(())
}
