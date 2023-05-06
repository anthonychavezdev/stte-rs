use crate::buffer::Buffer;
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, terminal, style, };
use std::io::{stdout, Stdout, Write};

/// The Screen struct represents the terminal screen.
pub struct Screen {
    win_size: Result<(u16, u16), crossterm::ErrorKind>,
    stdout: Stdout
}

impl Screen {
    pub fn new() -> Self {
        let win_size = terminal::size().map(|(x, y)| (x as u16, y as u16));
        let stdout = stdout();
        Self {
            win_size,
            stdout,
        }
    }

    fn draw_eof_indicators(&mut self, starting_row: u16) -> crossterm::Result<()> {
        if let Ok(win_size) = self.win_size {
            let (_, screen_rows) = win_size;
            for i in starting_row..screen_rows - 1 {
                queue!(self.stdout, cursor::MoveTo(0, i),
                    style::SetForegroundColor(style::Color::DarkGrey),
                    style::Print("~"))?;
                if i < screen_rows - 1 {
                    queue!(self.stdout, style::Print("\r\n"))?;
                }
            }
        }
        queue!(self.stdout, style::ResetColor)?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn clear() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), terminal::Clear(ClearType::Purge))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    pub fn refresh(&mut self) -> crossterm::Result<()> {
        execute!(self.stdout, terminal::Clear(ClearType::CurrentLine))?;
        Ok(())
    }
    pub fn display_buffer(&mut self, buffer: &Buffer) -> crossterm::Result<()> {
        self.refresh()?;
        let mut row: u16 = 0;

        for line in buffer.lines() {
            execute!(self.stdout, cursor::MoveTo(0, row),)?;
            self.stdout.write_all(line.to_string().as_bytes())?;
            self.stdout.write_all(b"\r\n")?;
            row += 1;
        }
        self.draw_eof_indicators(row)?;
        let (cursor_x, cursor_y) = buffer.get_cursor_xy();
        execute!(self.stdout, cursor::MoveTo(cursor_x as u16, cursor_y as u16))
    }

    pub fn display_status_message(&mut self, message: &str) -> crossterm::Result<()> {
        if let Ok(win_size) = self.win_size {
            let (_, screen_rows) = win_size;
            queue!(self.stdout,
                cursor::MoveTo(1, screen_rows - 1),
                terminal::Clear(ClearType::CurrentLine),
                style::Print(message))?;

            execute!(self.stdout,
            cursor::MoveTo(1, screen_rows - 2),
            terminal::Clear(ClearType::CurrentLine))?;
            self.stdout.flush()?;

        }
        Ok(())
    }
}
