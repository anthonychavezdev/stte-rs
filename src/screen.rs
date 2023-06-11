use crate::buffer::Buffer;
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, terminal, style, };
use std::io::{stdout, Stdout, Write};
use unicode_width::UnicodeWidthChar;

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

    pub fn window_size(&self) -> Result<(u16, u16), String> {
        self.win_size.as_ref().map_err(|e| format!("Error obtaining screen size: {}", e.to_string())).copied()
    }

    pub fn update_window_size(&mut self, width: u16, height: u16) -> crossterm::Result<()> {
        self.win_size = Ok((width, height));
        self.refresh()?;
        Ok(())
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

    pub fn clear(rows: u16, cols: u16) -> crossterm::Result<()> {
        // This avoids clearing the status line
        let status_message_row: u16 = rows - 1;
        execute!(stdout(), cursor::MoveTo(0, status_message_row))?;
        execute!(stdout(), terminal::Clear(ClearType::FromCursorUp))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    pub fn refresh_line(&mut self) -> crossterm::Result<()> {
        execute!(self.stdout, terminal::Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn refresh(&mut self) -> crossterm::Result<()> {
        if let Ok(win_size) = self.win_size {
            Screen::clear(win_size.0, win_size.1)?;
        }
        Ok(())
    }

    pub fn display_buffer(&mut self, buffer: &Buffer) -> crossterm::Result<()> {
        self.refresh_line()?;
        let mut row: u16 = 0;

        for line in buffer.lines() {
            queue!(self.stdout, cursor::MoveTo(0, row),)?;
            self.stdout.write_all(line.to_string().as_bytes())?;
            self.stdout.write_all(b"\r\n")?;
            row += 1;
        }
        self.draw_eof_indicators(row)?;
        let (_, cursor_y) = buffer.get_cursor_xy();
        // Some characters take up multiple culumns, so the cursor's x position needs
        // to be adjusted so it remains in sync with the the cursor in the rope data structure.
        let adjusted_cursor_x = buffer.get_visual_char_len();

        // This will perform poorly on long lines
        execute!(self.stdout, cursor::MoveTo(adjusted_cursor_x as u16, cursor_y as u16))
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
