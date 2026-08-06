use std::{env, io::{self}, panic, path::PathBuf};
use crossterm::{ExecutableCommand, execute, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};

use crate::{buffer::Buffer, display::Display, events::Action};

mod buffer;
mod cursor;
mod events;
mod display;

/** The `CleanUp` struct is used to disable raw_mode
when the struct goes out of scope.
It does this by implementing the `Drop` trait
and disabling raw_mode in the drop method.
This prevents the terminal from remaining in raw mode
if an error occurs after it's been set to raw mode
and the program exits. */
struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        execute!(io::stdout(), LeaveAlternateScreen).expect("error leaving alternate screen. Run the 'reset' command in your terminal");
        terminal::disable_raw_mode().expect("Could not disable raw mode. Run the 'reset' command in your terminal.");
    }
}

fn run(buffer: &mut Buffer) -> io::Result<()> {
    let mut display = Display::new()?;
    display.render(buffer)?;
    loop {
        match events::handle_events(&mut display)? {
            Action::Quit => return Ok(()),
            Action::Redraw => display.render(buffer)?,
            Action::Idle => {},
        }
    }
}

fn main() -> io::Result<()> {
    let path = env::args().nth(1).map(PathBuf::from);
    let mut buffer = Buffer::new(path)?;
    let mut stdout = io::stdout();
    let _cleanup = Cleanup;
    stdout.execute(EnterAlternateScreen).expect("Error entering alternative screen");
    terminal::enable_raw_mode()?;
    run(&mut buffer)
}

