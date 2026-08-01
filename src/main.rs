use std::{io, thread, time::Duration};

use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen}};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).expect("Error entering alternative screen");
    execute!(stdout, LeaveAlternateScreen)?;
    return Ok(())
}
