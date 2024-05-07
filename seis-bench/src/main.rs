//! This is a highly-parallelized benchmarking software
mod cli;
mod config;
mod results;

use crate::cli::Cli;
use clap::Parser;
use config::{Benchmark, SimulationConfig};
use crossterm::{
    cursor::{Hide, MoveToColumn, MoveToPreviousLine, RestorePosition, SavePosition, Show},
    execute,
    style::{StyledContent, Stylize},
};
use libmem::memory::Memory;
use libpipe::ClockResult;
use libseis::{pages::PAGE_SIZE, types::Word};
use rayon::prelude::*;
use results::RunResult;
use std::{
    error::Error,
    fs::File,
    io::{stdout, Write},
    sync::Mutex,
    time::Instant,
};

/// The maximum number of pages allocated for each instance of the simulation.
///
/// This may not ever be reached by a given benchmark.
const PAGES: usize = 16;

/// Calls the assembler to build the binary for the benchmark.
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

/// Prepares a simulation. Loads benchmark to memory.
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

/// Runs a benchmark with a given configuration.
///
/// The `benchmark` passed will be run with a provided `config`.
fn run_benchmark<'a>(
    benchmark: &'a Benchmark,
    config: &'a SimulationConfig,
) -> Result<RunResult, Box<dyn Error>> {
    let mut pipeline = config.build_config();
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
        bench_name: benchmark.name.clone(),
        config_name: config.name.clone(),
        clocks,
        rtc: end - start,
    })
}

const STATUS_WIDTH: usize = 12;

/// Formats the text for the "processing" status. Sets color to cyan and makes
/// the text boldface. It also right-aligns the text within a `STATUS_WIDTH` field.
fn processing_status(text: &str) -> StyledContent<String> {
    format!("{text:>STATUS_WIDTH$}").cyan().bold()
}

/// Formats the text for the "finished" status. Sets color to green and makes
/// the text boldface. It also right-aligns the text within a `STATUS_WIDTH` field.
fn finished_status(text: &str) -> StyledContent<String> {
    format!("{text:>STATUS_WIDTH$}").green().bold()
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut config = config::read_configuration(&cli.bench_conf)?;

    // Ensure there is at *least* one benchmark and one configuration.
    if config.benchmark.len() == 0 {
        println!("There are no benchmarks to run.");
        return Ok(());
    }

    if config.configuration.len() == 0 {
        println!("There are no configurations to use.");
        return Ok(());
    }

    // Hide the cursor
    execute!(stdout(), Hide)?;

    // Get the width of the widest field.
    let bench_name_width = config.benchmark.iter().map(|b| b.name.len()).max().unwrap();
    let config_name_width = config
        .configuration
        .iter()
        .map(|c| c.name.len())
        .max()
        .unwrap();

    // Get the path of the configuration file and set the paths
    // of each benchmark correctly before building each binary.
    //
    // This is done in series to prevent output squashing.
    let conf_path = cli.bench_conf.parent().unwrap();
    config
        .benchmark
        .iter_mut()
        .try_for_each(|b| -> Result<(), Box<dyn Error>> {
            print!(
                "{} benchmark {}",
                processing_status("Building"),
                format!("{:>bench_name_width$}", b.name).italic()
            );
            stdout().flush()?;

            b.path = conf_path.join(&b.path);
            build_binary(b)?;

            execute!(stdout(), MoveToColumn(0))?;
            println!(
                "{} benchmark {}",
                finished_status("Built"),
                format!("{:>bench_name_width$}", b.name).italic()
            );

            Ok(())
        })?;

    // Get every combination of configuration and benchmark.
    let configurations: Vec<_> = config
        .benchmark
        .iter()
        .flat_map(|bench| config.configuration.iter().map(move |conf| (bench, conf)))
        .collect();

    // Move to the previous line (as all runs will add a newline before printing text)
    execute!(stdout(), MoveToPreviousLine(1))?;
    stdout().flush()?;

    // Set the thread count accordingly.
    if let Some(threads) = cli.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()?;
    } else {
        rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build_global()?;
    }

    // Count the number of finished runs
    // This is a mutex to enable multithreaded use
    let n = Mutex::new(0u16);

    // Run each combination of benchmark and configuration.
    //
    // There is still an issue where benchmarks will go out of bounds of the console,
    // but that is not of significant concern at the moment.
    let results: Vec<_> = configurations
        .into_par_iter()
        .flat_map(|(benchmark, config)| -> Result<RunResult, Box<dyn Error>> {
            let mut lock = stdout().lock();
            let mut n_lock = n.lock().unwrap();
            let i = *n_lock;
            *n_lock += 1;
            write!(
                lock,
                "\n{} benchmark {} {:<config_name_width$}",
                processing_status("Running"),
                format!("{:>bench_name_width$}", benchmark.name).italic(),
                config.name
            )?;
            lock.flush()?;
            drop((lock, n_lock));

            let run = run_benchmark(benchmark, config)?;

            let mut lock = stdout().lock();
            let n_lock = n.lock().unwrap();
            execute!(lock, SavePosition)?;
            execute!(lock, MoveToPreviousLine(*n_lock - i))?;
            write!(
                lock,
                "\n{} benchmark {} {:<config_name_width$}; took {:.2} seconds",
                finished_status("Finished"),
                format!("{:>bench_name_width$}", benchmark.name).italic(),
                config.name,
                run.rtc.as_secs_f64(),
            )?;
            execute!(lock, RestorePosition)?;

            Ok(run)
        })
        .collect();

    println!(
        "\n{} (took {:.2} seconds)",
        finished_status("Done"),
        results.iter().fold(0.0, |a, r| a + r.rtc.as_secs_f64())
    );

    let file = cli.output_file();
    println!(
        "{} results to {}...",
        finished_status("Writing"),
        file.display()
    );

    let mut file = File::create(file)?;

    writeln!(file, "benchmark,configuration,clocks,rtc")?;
    results
        .into_iter()
        .try_for_each(|line| writeln!(file, "{}", line))?;

    execute!(stdout(), Show)?;

    Ok(())
}
