use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState};
use crossterm::terminal;
use std::env;
use std::fs::File;

use buffer::Buffer;
use screen::Screen;

mod buffer;
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

struct TextEditor<'a> {
    buffer: Option<Buffer>,
    output: Screen<'a>,
    reader: keyboard::KeyboardReader,
}

impl<'a> TextEditor<'a> {
    fn new() -> Self {
        Self {
            reader: keyboard::KeyboardReader,
            output: Screen::new(),
            buffer: None,
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

    fn run(&mut self, buffer: &'a Option<Buffer>) -> crossterm::Result<bool> {
        self.output.display_buffer(&buffer)?;
        self.process_keypress()
    }
}

fn main() -> crossterm::Result<()> {
    // When this variable goes out of scope the drop method is ran
    let _clean_up: CleanUp = CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor: TextEditor = TextEditor::new();
    let args: Vec<String> = env::args().collect();

    let buffer: Option<Buffer> = if args.len() > 1 {
        let path = &args[1];
        let file = File::open(path)?;
        match Buffer::from_path(&path) {
            Ok(buffer) => Some(buffer),
            Err(error) => {
                eprintln!("Error creating buffer or opening file\n{:?}", error);
                None
            }
        }
    } else {
        None
    };
    while editor.run(&buffer)? {}
    Ok(())
}
