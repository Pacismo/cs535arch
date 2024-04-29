mod cli;
mod config;
mod results;

use crate::cli::Cli;
use clap::Parser;
use config::{Benchmark, SimulationConfig};
use crossterm::{
    cursor::{Hide, MoveToColumn, Show},
    execute,
    style::Stylize,
};
use libmem::memory::Memory;
use libpipe::ClockResult;
use libseis::{pages::PAGE_SIZE, types::Word};
use results::RunResult;
use std::{
    error::Error,
    fs::File,
    io::{stdout, Write},
    time::Instant,
};

const PAGES: usize = 16;

pub fn build_binary(benchmark: &Benchmark) -> Result<(), Box<dyn Error>> {
    use std::process::Command;

    let status = Command::new("seis-asm")
        .args(&benchmark.sources)
        .arg("-o")
        .arg(&benchmark.binary)
        .current_dir(&benchmark.path)
        .spawn()?
        .wait()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Failed to build file (process exited with code {status})").into())
    }
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

    if config.benchmark.len() == 0 {
        println!("There are no benchmarks to run.");
        return Ok(());
    }

    execute!(stdout(), Hide)?;
    println!("  {} benchmarks...", "Building".green().bold());

    let name_width = config.benchmark.iter().map(|b| b.name.len()).max().unwrap();

    let conf_path = cli.bench_conf.parent().unwrap();
    config
        .benchmark
        .iter_mut()
        .try_for_each(|b| -> Result<(), Box<dyn Error>> {
            print!(
                "  {} benchmark {}",
                "Building".bold().cyan(),
                format!("{:>name_width$}", b.name).italic()
            );
            stdout().flush()?;

            b.path = conf_path.join(&b.path);
            build_binary(b)?;

            execute!(stdout(), MoveToColumn(0))?;
            println!(
                "  {} benchmark {}",
                "   Built".bold().green(),
                format!("{:>name_width$}", b.name).italic()
            );

            Ok(())
        })?;

    println!("      {}", "Done".bold().green());

    let mut results = vec![];
    results.reserve(config.benchmark.len() * 4);

    println!("   {} benchmarks...", "Running".green().bold());

    for (benchmark, pipeline, cache) in config.benchmark.iter().flat_map(|bench| {
        [
            (bench, false, false),
            (bench, false, true),
            (bench, true, false),
            (bench, true, true),
        ]
    }) {
        print!(
            "  {} benchmark {} ({}, {})",
            " Running".bold().cyan(),
            format!("{:>name_width$}", benchmark.name).italic(),
            if pipeline {
                "pipeline".green()
            } else {
                "pipeline".red()
            },
            if cache {
                "cache".green()
            } else {
                "cache".red()
            },
        );
        stdout().flush()?;

        let run = run_benchmark(benchmark, &config.configuration, pipeline, cache)?;

        execute!(stdout(), MoveToColumn(0))?;
        println!(
            "  {} benchmark {} ({}, {}); took {:.2} seconds",
            "Finished".bold().green(),
            format!("{:>name_width$}", benchmark.name).italic(),
            if pipeline {
                "pipeline".green()
            } else {
                "pipeline".red()
            },
            if cache {
                "cache".green()
            } else {
                "cache".red()
            },
            run.rtc.as_secs_f64(),
        );

        results.push(run);
    }

    println!(
        "      {} (took {:.2} seconds)",
        "Done".bold().green(),
        results.iter().fold(0.0, |a, r| a + r.rtc.as_secs_f64())
    );

    let file = cli.output_file();
    println!("Writing results to {}...", file.display());

    let mut file = File::create(file)?;

    writeln!(file, "name, pipeline, cache, clocks, rtc")?;
    results
        .into_iter()
        .try_for_each(|line| writeln!(file, "{}", line))?;

    execute!(stdout(), Show)?;

    Ok(())
}
