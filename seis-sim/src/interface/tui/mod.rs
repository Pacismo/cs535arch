use super::Interface;
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct Tui;

impl Interface for Tui {
    type Ok = ();

    type Error = Box<dyn Error>;

    fn run(self, pipeline: Box<dyn libpipe::Pipeline>) -> Result<Self::Ok, Self::Error> {
        // Initialize the terminal

        loop {
            // Handle input events and draw UI
            todo!("Implement TUI")
        }

        // Restore terminal state
    }
}
