use std::{io::{self}};
use crossterm::{ExecutableCommand, execute, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};

use crate::{events::Action};

mod events;

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

fn run() -> io::Result<()> {
    loop {
        match events::handle_events()? {
            Action::Quit => return Ok(()),
            Action::Idle => {}
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let _cleanup = Cleanup;
    stdout.execute(EnterAlternateScreen).expect("Error entering alternative screen");
    terminal::enable_raw_mode()?;
    run()
}

