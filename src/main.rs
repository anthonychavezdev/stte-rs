use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState};
use crossterm::terminal;
use std::env;

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
        Screen::clear().expect("Error");
        terminal::disable_raw_mode().expect("Could not turn off raw mode");
    }
}

struct TextEditor {
    output: Screen,
    reader: keyboard::KeyboardReader,
}

impl TextEditor {
    fn new() -> Self {
        Self {
            reader: keyboard::KeyboardReader,
            output: Screen::new(),
        }
    }

    fn process_keypress(&self, buffer: &mut Buffer) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => return Ok(false),
            KeyEvent {
                code: KeyCode::Left,
                modifiers: event::KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => {
                buffer.move_cursor_left();
            }
            KeyEvent {
                code: KeyCode::Right,
                modifiers: event::KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => {
                buffer.move_cursor_right();
            }
            KeyEvent {
                code: KeyCode::Up,
                modifiers: event::KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => {
                buffer.move_cursor_up();
            }
            KeyEvent {
                code: KeyCode::Down,
                modifiers: event::KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => {
                buffer.move_cursor_down();
            }
            _ => {}
        }
        Ok(true)
    }

    fn run(&mut self, buffer: &mut Buffer) -> crossterm::Result<bool> {
        self.output.display_buffer(&buffer)?;
        self.process_keypress(buffer)
    }
}

fn main() -> crossterm::Result<()> {
    // When this variable goes out of scope the drop method is ran
    let _clean_up: CleanUp = CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor: TextEditor = TextEditor::new();
    let args: Vec<String> = env::args().collect();

    let mut buffer: Buffer = if args.len() > 1 {
        let path: &String = &args[1];
        match Buffer::from_path(&path) {
            Ok(buffer) => buffer,
            Err(error) => {
                eprintln!("Error creating buffer or opening file\n{:?}", error);
                Buffer::new() // Create an empty buffer if there's an error
            }
        }
    } else {
        Buffer::new() // Create an empty buffer if no file is specified
    };
    while editor.run(&mut buffer)? {}
    Ok(())
}
