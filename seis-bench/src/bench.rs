use crate::{
    config::{Benchmark, SimulationConfig},
    error::Error,
    results::RunResult,
    run_benchmark,
};
use rayon::prelude::*;
use std::{
    sync::mpsc::{channel, Receiver},
    thread::{spawn, JoinHandle},
};

pub enum State {
    Started(usize),
    Finished(usize),
}
use State::*;

pub struct BenchmarkHelper(JoinHandle<Result<Vec<RunResult>, Error>>, Receiver<State>);

impl BenchmarkHelper {
    pub fn new<
        T: Send
            + Sync
            + IntoParallelIterator<
                Item = (usize, impl AsRef<Benchmark>, impl AsRef<SimulationConfig>),
            > + 'static,
    >(
        configurations: T,
        n: usize,
    ) -> std::thread::Result<Self> {
        let (tx, rx) = channel();

        let thread = spawn(move || {
            rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build_global()?;

            configurations
                .into_par_iter()
                .map(|(i, bench, conf)| {
                    tx.send(Started(i))?;
                    let run = run_benchmark(bench.as_ref(), conf.as_ref())?;
                    tx.send(Finished(i))?;
                    Ok(run)
                })
                .collect()
        });

        Ok(Self(thread, rx))
    }

    pub fn next(&self) -> Option<State> {
        self.1.recv().ok()
    }

    pub fn join(self) -> Result<Vec<RunResult>, Error> {
        self.0.join().unwrap()
    }
}
