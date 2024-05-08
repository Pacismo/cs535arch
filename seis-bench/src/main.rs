//! This is a highly-parallelized benchmarking software
mod bench;
mod cli;
mod config;
mod error;
mod results;

use crate::{bench::BenchmarkHelper, cli::Cli, error::Error};
use clap::Parser;
use config::{Benchmark, SimulationConfig};
use crossterm::{
    cursor::{Hide, MoveToColumn, MoveToPreviousLine, Show},
    execute,
    style::{StyledContent, Stylize},
};
use libmem::memory::Memory;
use libpipe::ClockResult;
use libseis::{pages::PAGE_SIZE, types::Word};
use results::RunResult;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{stdout, Write},
    sync::Arc,
    time::Instant,
};

/// The maximum number of pages allocated for each instance of the simulation.
///
/// This may not ever be reached by a given benchmark.
const PAGES: usize = 16;

/// Calls the assembler to build the binary for the benchmark.
pub fn build_binary(benchmark: &Benchmark) -> Result<(), Error> {
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
pub fn prepare_sim(mem: &mut Memory, benchmark: &Benchmark) -> Result<(), Error> {
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
) -> Result<RunResult, Error> {
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

/// How wide to make the status field of a line
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

fn run<'a: 'static>(
    n: usize,
    configurations: Vec<(usize, Arc<Benchmark>, Arc<SimulationConfig>)>,
) -> Result<Vec<RunResult>, Error> {
    let (bench_width, conf_width) = configurations
        .iter()
        .map(|(_, b, c)| (b.name.len(), c.name.len()))
        .reduce(|acc, e| (acc.0.max(e.0), acc.1.max(e.1)))
        .unwrap();

    let mappings: HashMap<usize, (Arc<Benchmark>, Arc<SimulationConfig>)> = configurations
        .iter()
        .map(|(i, bench, conf)| (*i, (bench.clone(), conf.clone())))
        .collect();

    // Run each combination of benchmark and configuration.
    //
    // There is still an issue where benchmarks will go out of bounds of the console,
    // but that is not of significant concern at the moment.
    let helper = BenchmarkHelper::new(configurations, n)?;
    let mut running = HashSet::new();

    while let Some(state) = helper.next() {
        match state {
            bench::State::Started(i) => {
                execute!(stdout(), MoveToPreviousLine(running.len().max(1) as u16))?;
                running.insert(i);

                for i in running.iter() {
                    let (bench, conf) = mappings.get(i).unwrap();
                    print!(
                        "\n{} {:>bench_width$} {:<conf_width$}",
                        processing_status("Running"),
                        bench.name,
                        conf.name
                    )
                }
            }
            bench::State::Finished(i) => {
                let (bench, conf) = mappings.get(&i).unwrap();
                execute!(stdout(), MoveToPreviousLine(running.len() as u16))?;
                print!(
                    "\n{} {:>bench_width$} {:<conf_width$}",
                    finished_status("Finished"),
                    bench.name,
                    conf.name
                );
                running.remove(&i);

                for i in running.iter() {
                    let (bench, conf) = mappings.get(i).unwrap();
                    print!(
                        "\n{} {:>bench_width$} {:<conf_width$}",
                        processing_status("Running"),
                        bench.name,
                        conf.name
                    )
                }
            }
        }
        stdout().flush()?;
    }

    helper.join()
}

fn main() -> Result<(), Error> {
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

    // Get the path of the configuration file and set the paths
    // of each benchmark correctly before building each binary.
    //
    // This is done in series to prevent output squashing.
    let conf_path = cli.bench_conf.parent().unwrap();
    config
        .benchmark
        .iter_mut()
        .try_for_each(|b| -> Result<(), Error> {
            print!(
                "{} benchmark {}",
                processing_status("Building"),
                b.name.as_str().italic()
            );
            stdout().flush()?;

            b.path = conf_path.join(&b.path);
            build_binary(b)?;

            execute!(stdout(), MoveToColumn(0))?;
            println!(
                "{} benchmark {}",
                finished_status("Built"),
                b.name.as_str().italic()
            );

            Ok(())
        })?;

    // Get every combination of configuration and benchmark.
    let conf = config
        .configuration
        .into_iter()
        .map(Arc::new)
        .collect::<Vec<_>>();
    let configurations: Vec<_> = config
        .benchmark
        .into_iter()
        .flat_map(|bench| {
            let bench = Arc::new(bench);

            conf.iter().map(move |conf| (bench.clone(), conf.clone()))
        })
        .enumerate()
        .map(|(i, (b, c))| (i, b, c))
        .collect();

    // Move to the previous line (as all runs will add a newline before printing text)
    stdout().flush()?;

    let n = cli.threads.unwrap_or(4);

    let results = run(n, configurations)?;

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

    RunResult::write_headers(&mut file)?;
    results
        .into_iter()
        .try_for_each(|line| writeln!(file, "{}", line))?;

    execute!(stdout(), Show)?;

    Ok(())
}
