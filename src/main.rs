use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState};
use crossterm::terminal;
use screen::Screen;

mod keyboard;
mod screen;

/** The `CleanUp` struct is used to disable raw_mode
when the struct goes out of scope.
It does this by implementing the `Drop` trait
and disabling raw_mode in the drop method.
This prevents the terminal from remaining in raw mode
if an error occurs after it's been set to raw mode
and the program exits. */
struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not turn off raw mode");
        Screen::clear().expect("Error");
    }
}

struct Editor {
    reader: keyboard::Reader,
    output: Screen,
}

impl Editor {
    fn new() -> Self {
        Self {
            reader: keyboard::Reader,
            output: Screen::new(),
        }
    }
    fn process_keypress(&self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => return Ok(false),
            _ => {}
        }
        Ok(true)
    }

    fn run(&self) -> crossterm::Result<bool> {
        self.output.refresh()?;
        self.process_keypress()
    }
}

fn main() -> crossterm::Result<()> {
    // When this variable goes out of scope the drop method is ran
    let _clean_up: CleanUp = CleanUp;
    terminal::enable_raw_mode()?;
    let editor: Editor = Editor::new();
    while editor.run()? {}
    Ok(())
}
