mod init;

use self::init::{deinitialize, initialize};

use super::Interface;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use std::error::Error;
use std::io::{stdout, Stdout};

#[derive(Debug, Clone, Copy)]
pub struct Tui;

impl Interface for Tui {
    type Ok = ();

    type Error = Box<dyn Error>;

    fn run(self, _pipeline: Box<dyn libpipe::Pipeline>) -> Result<Self::Ok, Self::Error> {
        // Initialize the terminal
        let mut terminal = initialize()?;

        loop {
            // Handle input events, update pipeline, and draw UI
            println!("Implement TUI");

            terminal.draw(|frame| {
                let area = frame.size();
                frame.render_widget(Paragraph::new("Hello, World!").white().on_blue(), area);
            })?;

            break;
        }

        // Restore terminal state
        deinitialize()?;

        Ok(())
    }
}
