mod init;
mod ui;

use libpipe::Pipeline;

use crate::config::SimulationConfiguration;

use self::{
    init::{deinitialize, initialize},
    ui::Runtime,
};
use super::Interface;
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct Tui;

impl Interface for Tui {
    type Ok = ();

    type Error = Box<dyn Error>;

    fn run(self, mut pipeline: Box<dyn Pipeline>, config: SimulationConfiguration) -> Result<Self::Ok, Self::Error> {
        // Initialize the terminal
        let mut terminal = initialize()?;

        let mut runtime = Runtime::new(pipeline.as_mut(), config);
        let result = runtime.run(&mut terminal);

        // Restore terminal state
        deinitialize()?;

        result
    }
}
