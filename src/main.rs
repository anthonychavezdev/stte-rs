use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event, terminal, execute};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState};
use std::env;
use std::io::stdout;
use std::path::PathBuf;

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
        execute!(stdout(), LeaveAlternateScreen).unwrap();
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
                match buffer.save() {
                    Ok(message) => {
                        self.output.display_status_message(&message)?;
                    }
                    Err(e) => {
                        self.output.display_status_message(&format!("{}", e.to_string()))?;
                    }
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
                buffer.delete_char()?;
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
    // Enter the alternate screen buffer
    execute!(stdout(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    let mut editor: TextEditor = TextEditor::new();
    let args: Vec<String> = env::args().collect();
    let mut buffer: Buffer = if args.len() > 1 {
        let path: &String = &args[1];
        match Buffer::from_path(&path) {
            Ok(buffer) => buffer,
            Err(error) => {
                editor.output.display_status_message(&error.to_string())?;
                Buffer::new(Some(PathBuf::from(&path))) // Create a buffer if there's an error but a path is still provided
            }
        }
    } else {
        Buffer::new(None) // Create an empty buffer if no file is specified
    };
    // Clear terminal screen on first run
    let window_size = editor.output.window_size();
    match window_size {
        Ok((r, c)) => {
            Screen::clear(r, c)?;
        }
        Err(e) => {
            execute!(stdout(), LeaveAlternateScreen).unwrap();
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
    while editor.run(&mut buffer)? {}
    Ok(())
}
