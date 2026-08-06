use std::io::{self, Stdout, Write};

use crossterm::{QueueableCommand, style, terminal::{self, Clear, ClearType::{self}}};
use unicode_segmentation::UnicodeSegmentation;

use crate::buffer::Buffer;
use crate::cursor;

pub struct Display {
    stdout: Stdout,
    width: u16,
    height: u16,
    line_buffer: String
}

impl Display {
    pub fn new() -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self {
            stdout: io::stdout(),
            width,
            height,
            line_buffer: String::new()
        })
    }

    pub fn set_dimensions(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    pub fn height(&self) -> usize {
        self.height as usize
    }

    fn fill_line_buffer(&mut self, line: &str, width: usize) -> io::Result<()> {
        self.line_buffer.clear();
        let max = width;
        let mut col = 0;
        for grapheme in line.graphemes(true) {
            let width = cursor::grapheme_width(grapheme, col);
            if col + width > max {
                break;
            }
            if grapheme == "\t" {
                for _ in 0..width {
                    self.line_buffer.push(' ');
                }
            } else {
                self.line_buffer.push_str(grapheme);
            }
            col += width
        }
        Ok(())
    }


    pub fn render(&mut self, buffer: &Buffer) -> io::Result<()> {
        self.stdout.queue(crossterm::cursor::Hide)?;
        let scroll_offset = buffer.scroll_offset();
        let rope = buffer.rope();
        let total_lines = rope.len_lines();
        let width = self.width as usize;
        for row in 0..self.height {
            self.stdout.queue(crossterm::cursor::MoveTo(0, row))?;
            let line_idx = scroll_offset + row as usize;
            if line_idx < total_lines {
                let line = buffer.line(line_idx);
                self.fill_line_buffer(&line, width)?;
                self.stdout.queue(style::Print(&self.line_buffer))?;
            }
            self.stdout.queue(Clear(ClearType::UntilNewLine))?;
        }
        let cursor = buffer.cursor();
        let line = buffer.line(cursor.line_indx());
        let col = cursor::display_width(&line[..cursor.col()])
            .min(self.width.saturating_sub(1) as usize);
        let row = (cursor.line_indx() - scroll_offset).min(self.height().saturating_sub(1) as usize);
        self.stdout.queue( crossterm::cursor::MoveTo(col as u16, row as u16))?
            .queue(crossterm::cursor::Show)?;
        self.stdout.flush()
    }
}
