use std::ops::Deref;

use super::*;

pub fn initialize() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = stdout();

    stdout.execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;

    Ok(terminal)
}

pub fn deinitialize() -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();

    stdout.execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
