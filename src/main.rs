use crossterm::{event, terminal};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState};
use std::env;

use buffer::Buffer;
use screen::Screen;

mod buffer;
mod keyboard;
mod screen;
mod file_props;

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

    fn process_keypress(&mut self, buffer: &mut Buffer) -> crossterm::Result<bool> {
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
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: event::KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => {
                if let Err(e) = buffer.save() {
                    eprintln!("Error saving file: {:?}", e);
                } else {
                    println!("File saved successfully.");
                }
            }
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: _,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE
            } => {
                buffer.insert_newline()?;
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE
            } => {
                if modifiers.contains(event::KeyModifiers::SHIFT) {
                    buffer.insert_char(c.to_uppercase().next().unwrap_or(c));
                } else {
                    buffer.insert_char(c);
                }
            }
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: event::KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE
            } => {
                buffer.delete_char();
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
                Buffer::new(None) // Create an empty buffer if there's an error
            }
        }
    } else {
        Buffer::new(None) // Create an empty buffer if no file is specified
    };
    // Clear terminal screen on first run
    Screen::clear().expect("Error clearing screen");
    while editor.run(&mut buffer)? {}
    Ok(())
}
