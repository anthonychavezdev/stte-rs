use std::{borrow::Cow, fs::File, io::{self, BufReader}, path::PathBuf};

use ropey::Rope;

use crate::cursor::Cursor;

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
}
