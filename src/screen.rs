use crate::buffer::Buffer;
use crossterm::style::{Color, Print, SetForegroundColor};
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, terminal};
use std::io::{stdout, Write};

/// The Screen struct represents the terminal screen, with its size and optional reference to a buffer.
pub struct Screen<'a> {
    win_size: Result<(u16, u16), crossterm::ErrorKind>,
    buffer: Option<&'a Buffer>
}

impl<'a> Screen<'a> {
    pub fn new() -> Self {
        let win_size = terminal::size().map(|(x, y)| (x as u16, y as u16));
        Self {
            win_size,
            buffer: None
        }
    }

    fn draw_eof_indicators(&self, starting_row: u16) -> crossterm::Result<()> {
        let mut stdout = stdout();
        if let Ok((_, screen_rows)) = self.win_size {
            for i in starting_row..screen_rows {
                queue!(stdout, cursor::MoveTo(0, i), Print("~"))?;
                if i < screen_rows - 1 {
                    queue!(stdout, Print("\r\n"))?;
                }
            }
        }
        stdout.flush()?;
        Ok(())
    }

    pub fn clear() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    pub fn refresh(&self) -> crossterm::Result<()> {
        Self::clear()?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }
    pub fn display_buffer(&mut self, buffer: &'a Option<Buffer>) -> crossterm::Result<()> {
        self.buffer = buffer.as_ref();
        self.refresh()?;
        let mut row: u16 = 0;
        let mut output = stdout();
        if let Some(buffer) = self.buffer {
            for line in buffer.lines() {
                execute!(output, cursor::MoveTo(0, row),)?;
                output.write_all(line.to_string().as_bytes())?;
                output.write_all(b"\r\n")?;
                row += 1;
            }
        }
        self.draw_eof_indicators(row)?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }
}
