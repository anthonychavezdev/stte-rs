use std::{borrow::Cow, fs::File, io::{self, BufReader}, path::PathBuf};

use ropey::Rope;

use crate::cursor::{self, Cursor, Direction};

pub struct Buffer {
    text: Rope,
    #[allow(dead_code)]
    path: Option<PathBuf>,
    cursor: Cursor,
    scroll_offset: usize
}

impl Buffer {
    pub fn new(path: Option<PathBuf>) -> io::Result<Buffer> {
        let text = match &path {
            Some(p) if p.exists() => {
                Rope::from_reader(BufReader::new(File::open(p)?))?
            }
            _ => Rope::new()
        };
        Ok(Buffer {
            text,
            path,
            cursor: Cursor::default(),
            scroll_offset: 0
        })
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn rope(&self) -> &Rope {
        &self.text
    }

    pub fn line(&self, idx: usize) -> Cow<'_, str> {
        if idx >= self.text.len_lines() {
            return Cow::from("");
        }
        let slice = self.text.line(idx);
        let mut end = slice.len_chars();
        if end > 0 && slice.char(end - 1) == '\n' {
            end -= 1;
        }
        if end > 0 && slice.char(end - 1) == '\r' {
            end -= 1;
        }
        Cow::from(slice.slice(..end))
    }

    pub fn move_up(&mut self) {
        let line_indx = self.cursor.line_indx();
        if line_indx > 0 {
            self.cursor.set_line_indx(line_indx - 1);
            self.snap_col_to_desired();
        }
    }

    pub fn move_right(&mut self) {
        let line_indx = self.cursor.line_indx();
        let line = self.line(line_indx);
        let line_len = self.line(line_indx).len();
        let col = self.cursor.col();
        if col < line_len {
            let new_col = cursor::next_grapheme_boundary(&line, col);
            drop(line);
            self.cursor.set_col(new_col);
        } else if line_indx + 1 < self.text.len_lines() {
            self.cursor.set_line_indx(line_indx + 1);
            self.cursor.set_col(0);
        }
        self.sync_desired_col();
    }

    pub fn move_down(&mut self) {
        let line_indx = self.cursor.line_indx();
        if line_indx + 1 < self.text.len_lines() {
            self.cursor.set_line_indx(line_indx + 1);
            self.snap_col_to_desired();
        }
    }

    pub fn move_left(&mut self) {
        let col = self.cursor.col();
        let mut line_indx = self.cursor.line_indx();
        if col > 0 {
            let current_line = self.line(line_indx);
            let new_col = cursor::prev_grapheme_boundary(&current_line, col);
            drop(current_line);
            self.cursor.set_col(new_col);
        } else if line_indx > 0 {
            line_indx -= 1;
            self.cursor.set_line_indx(line_indx);
            self.cursor.set_col(self.line(line_indx).len());
        }
        self.sync_desired_col();
    }

    pub fn move_cursor(&mut self, dir: Direction) {
        match dir {
            Direction::Up => self.move_up(),
            Direction::Right => self.move_right(),
            Direction::Down => self.move_down(),
            Direction::Left => self.move_left(),
        }
    }

    fn sync_desired_col(&mut self) {
        let line_indx = self.cursor.line_indx();
        let line = self.line(line_indx);
        let col = self.cursor.col();
        let d = cursor::display_width(&line[..col]);
        drop(line);
        self.cursor.set_desired_col(d);
    }

    fn snap_col_to_desired(&mut self) {
        let line_indx = self.cursor.line_indx();
        let line = self.line(line_indx);
        let col = self.cursor.desired_col();
        let c = cursor::byte_at_display_col(&line, col);
        drop(line);
        self.cursor.set_col(c);
    }

    pub fn scroll(&mut self, v_width: usize, v_height: usize) {
        if v_height == 0 {
            return;
        }
        let line_indx = self.cursor.line_indx();
        if line_indx < self.scroll_offset {
            self.scroll_offset = line_indx;
        } else if line_indx >= self.scroll_offset + v_height {
            self.scroll_offset = line_indx + 1 - v_height;
        }
    }
}
