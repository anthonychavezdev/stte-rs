use std::{env, io::{self}, panic, path::PathBuf};
use crossterm::{ExecutableCommand, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};

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

fn restore_terminal() {
    let _ = io::stdout().execute(LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        restore_terminal();
    }
}

fn run(buffer: &mut Buffer) -> io::Result<()> {
    let mut display = Display::new()?;
    display.render(buffer)?;
    loop {
        match events::get_event_actions(&mut display)? {
            Action::Quit => return Ok(()),
            Action::Redraw => {
                let (v_width, v_height) = display.size();
                buffer.scroll(v_width, v_height);
                display.render(buffer)?
            },
            Action::CursorMovement(dir) => {
                buffer.move_cursor(dir);
                display.clear_status_message()
            },
            Action::Save => match buffer.save() {
                    Ok(bytes) => display.set_status(format!("Wrote {} bytes to {}", bytes, buffer.path())),
                    Err(e) => display.set_status(e.to_string())
                }
            Action::Idle => {},
        }
        let (v_width, _) = display.size();
        buffer.scroll(v_width, display.status_message_boundary());
        display.render(buffer)?
    }
}

fn main() -> io::Result<()> {
    let path = env::args().nth(1).map(PathBuf::from);
    let mut buffer = Buffer::new(path)?;
    let mut stdout = io::stdout();
    // Leave the alternate screen *before* the default hook prints the panic,
    // otherwise the message is written to the alt screen and vanishes.
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    let _cleanup = Cleanup;
    stdout.execute(EnterAlternateScreen).expect("Error entering alternative screen");
    terminal::enable_raw_mode()?;
    run(&mut buffer)
}

