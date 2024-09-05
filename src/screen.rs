use crate::buffer::Buffer;
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, style, terminal};
use ropey::RopeSlice;
use std::io::{stdout, Stdout, Write};
use std::time::{self, Duration};
use unicode_width::UnicodeWidthChar;

const TAB_WIDTH: usize = 8;

pub struct WindowSize {
    pub width: u16,
    pub height: u16,
}

/// The Screen struct represents the terminal screen.
pub struct Screen {
    win_size: WindowSize,
    stdout: Stdout,
    scroll_offset: usize,
    status_message: Option<String>,
    status_message_time: time::Instant,
}

impl Screen {
    pub fn new() -> Self {
        let (width, height) = terminal::size().expect("Failed to get terminal size");
        Self {
            win_size: WindowSize { width, height },
            stdout: stdout(),
            scroll_offset: 0,
            status_message: None,
            status_message_time: time::Instant::now(),
        }
    }

    pub fn window_size(&self) -> &WindowSize {
        &self.win_size
    }

    pub fn update_window_size(&mut self, width: u16, height: u16) -> crossterm::Result<()> {
        self.win_size = WindowSize { width, height };
        self.refresh()
    }

    fn draw_eof_indicators(&mut self, start_row: usize) -> crossterm::Result<()> {
        for row in start_row..self.win_size.height.saturating_sub(1) as usize {
            queue!(
                self.stdout,
                cursor::MoveTo(0, row as u16),
                terminal::Clear(ClearType::CurrentLine),
                style::SetForegroundColor(style::Color::DarkGrey),
                style::Print("~"),
                style::ResetColor
            )?;
        }
        Ok(())
    }

    pub fn clear(&mut self) -> crossterm::Result<()> {
        queue!(self.stdout, terminal::Clear(ClearType::All))
    }

    pub fn refresh(&mut self) -> crossterm::Result<()> {
        execute!(
            self.stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )
    }

    pub fn display_buffer(&mut self, buffer: &Buffer) -> crossterm::Result<()> {
        self.update_scroll_offset(buffer);
        self.draw_lines(buffer)?;
        self.draw_status_bar(buffer)?;
        self.position_cursor(buffer)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn update_scroll_offset(&mut self, buffer: &Buffer) {
        let cursor_row = buffer.cursor_row();
        let viewport_height = self.win_size.height.saturating_sub(1) as usize;

        if cursor_row < self.scroll_offset {
            self.scroll_offset = cursor_row;
        } else if cursor_row >= self.scroll_offset + viewport_height {
            self.scroll_offset = cursor_row.saturating_sub(viewport_height).saturating_add(1);
        }
    }

    fn draw_lines(&mut self, buffer: &Buffer) -> crossterm::Result<()> {
        let viewport_height = self.win_size.height.saturating_sub(1) as usize;
        let visible_lines = buffer
            .lines()
            .skip(self.scroll_offset)
            .take(viewport_height);

        for (row, line) in visible_lines.enumerate() {
            queue!(self.stdout, cursor::MoveTo(0, row as u16))?;
            self.draw_line(&line)?;
        }

        self.draw_eof_indicators(buffer.lines().count().saturating_sub(self.scroll_offset))?;
        Ok(())
    }

    fn draw_line(&mut self, line: &RopeSlice) -> crossterm::Result<()> {
        let mut visual_col = 0;

        for ch in line.chars() {
            if visual_col >= self.win_size.width as usize {
                break;
            }

            match ch {
                '\t' => {
                    let spaces = TAB_WIDTH - (visual_col % TAB_WIDTH);
                    queue!(self.stdout, style::Print(" ".repeat(spaces)))?;
                    visual_col += spaces;
                }
                '\n' => break,
                _ => {
                    queue!(self.stdout, style::Print(ch))?;
                    visual_col += 1;
                }
            }
        }

        queue!(self.stdout, terminal::Clear(ClearType::UntilNewLine))
    }

    fn draw_status_bar(&mut self, buffer: &Buffer) -> crossterm::Result<()> {
        let status_row = self.win_size.height.saturating_sub(1);
        queue!(
            self.stdout,
            cursor::MoveTo(0, status_row),
            terminal::Clear(ClearType::CurrentLine),
            style::SetAttribute(style::Attribute::Reverse)
        )?;

        let file_name = buffer.file_path().map_or("[No Name]", |path| {
            path.to_str().unwrap_or("[Invalid Path]")
        });
        let cursor_info = format!("{}:{}", buffer.cursor_row() + 1, buffer.cursor_column() + 1);
        let status = format!("{} - {}", file_name, cursor_info);

        queue!(
            self.stdout,
            style::Print(status),
            style::SetAttribute(style::Attribute::Reset)
        )?;

        if let Some(message) = &self.status_message {
            if self.status_message_time.elapsed() < Duration::from_secs(3) {
                queue!(
                    self.stdout,
                    cursor::MoveTo(0, status_row.saturating_sub(1)),
                    terminal::Clear(ClearType::CurrentLine),
                    style::Print(message)
                )?;
            } else {
                self.status_message = None;
            }
        }

        Ok(())
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
        self.status_message_time = time::Instant::now();
    }

    fn position_cursor(&mut self, buffer: &Buffer) -> crossterm::Result<()> {
        let (_, cursor_y) = buffer.get_cursor_xy();
        let visual_cursor_x = buffer.get_visual_cursor_x();
        let screen_y = cursor_y.saturating_sub(self.scroll_offset) as u16;

        execute!(
            self.stdout,
            cursor::MoveTo(visual_cursor_x as u16, screen_y)
        )
    }
}
