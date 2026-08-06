use std::io::{self, Stdout, Write};

use crossterm::{QueueableCommand, style, terminal::{self, Clear, ClearType::{self}}};
use unicode_segmentation::UnicodeSegmentation;

use crate::buffer::{Buffer, ScrollOffset};
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

    pub fn size(&self) -> (usize, usize) {
        (self.width as usize, self.height as usize)
    }

    fn fill_line_buffer(&mut self, line: &str, scroll_offset: &ScrollOffset) {
        self.line_buffer.clear();
        if self.width == 0 {
            return;
        }
        let end = scroll_offset.x + self.width as usize;
        let mut col = 0;
        for grapheme in line.graphemes(true) {
            let grapheme_width = cursor::grapheme_width(grapheme, col);
            let (grapheme_start, grapheme_end) = (col, col + grapheme_width);
            col = grapheme_end;
            if grapheme_end <= scroll_offset.x {
                continue;
            }
            if grapheme_start >= end {
                break;
            }
            if grapheme == "\t" || grapheme_start < scroll_offset.x {
                let visible = grapheme_end.min(end) - grapheme_start.max(scroll_offset.x);
                for _ in 0..visible {
                    self.line_buffer.push(' ');
                }
            } else if grapheme_end > end {
                break;
            } else {
                self.line_buffer.push_str(grapheme);
            }
        }
    }

    pub fn render(&mut self, buffer: &Buffer) -> io::Result<()> {
        self.stdout.queue(crossterm::cursor::Hide)?;
        let scroll_offset = buffer.scroll_offset();
        let rope = buffer.rope();
        let total_lines = rope.len_lines();
        let (width, height) = self.size();
        for row in 0..self.height {
            self.stdout.queue(crossterm::cursor::MoveTo(0, row))?;
            let line_idx = scroll_offset.y + row as usize;
            if line_idx < total_lines {
                let line = buffer.line(line_idx);
                self.fill_line_buffer(&line, scroll_offset);
                self.stdout.queue(style::Print(&self.line_buffer))?;
            }
            self.stdout.queue(Clear(ClearType::UntilNewLine))?;
        }
        let cursor = buffer.cursor();
        let line = buffer.line(cursor.line_indx());
        let col = cursor::display_width(&line[..cursor.col()])
            .saturating_sub(scroll_offset.x)
            .min(width.saturating_sub(1));
        let row = cursor.line_indx()
            .saturating_sub(scroll_offset.y)
            .min(height.saturating_sub(1));
        self.stdout.queue( crossterm::cursor::MoveTo(col as u16, row as u16))?
            .queue(crossterm::cursor::Show)?;
        self.stdout.flush()
    }
}
