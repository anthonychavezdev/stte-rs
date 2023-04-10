use crossterm::style::{Color, Print, SetForegroundColor};
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, terminal};
use std::io::{stdout, Write};

/// The Screen struct represents the terminal screen, with its size.
pub struct Screen {
    win_size: Result<(u16, u16), crossterm::ErrorKind>,
}

impl Screen {
    pub fn new() -> Self {
        let win_size = terminal::size().map(|(x, y)| (x as u16, y as u16));
        Self {
            win_size,
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
        self.draw_eof_indicators()?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }
}
