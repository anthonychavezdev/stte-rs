use crossterm::terminal::ClearType;
use crossterm::{execute, terminal};
use ropey::iter::{Bytes, Chars, Chunks, Lines};
use ropey::{Rope, RopeSlice};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::{PathBuf, Path};

use crate::file_props::FileProps;

#[derive(Debug)]
pub enum Status {
    Modified,
    Clean,
    Saving,
}

#[derive(Debug)]
pub struct Buffer {
    text: Rope,                 // text from a file or in memory
    file_path: Option<PathBuf>, // path associated with a file. Buffers don't always need to be associated with a file, they can be in memory only
    status: Status, // Whether the buffer has been modified, left unchanged, or is being saved back to disk?
    cursor_pos: usize,
    file_props: Option<FileProps>
}

impl Buffer {
    pub fn new(file_props: Option<FileProps>) -> Buffer {
        let text = Rope::new();
        Buffer {
            text,
            file_path: None,
            status: Status::Clean,
            cursor_pos: 0,
            file_props
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.text.len_chars() {
            self.cursor_pos += 1;
        }
    }

    pub fn move_cursor_up(&mut self) {
        let (x, y) = self.get_cursor_xy();
        if y > 0 {
            let new_y = y - 1;
            let new_x = std::cmp::min(x, self.text.line(new_y).len_chars());
            self.cursor_pos = self.text.line_to_char(new_y) + new_x;
        }
    }

    pub fn move_cursor_down(&mut self) {
        let (x, y) = self.get_cursor_xy();
        if y < self.text.len_lines() - 1 {
            let new_y = y + 1;
            let new_x = std::cmp::min(x, self.text.line(new_y).len_chars());
            self.cursor_pos = self.text.line_to_char(new_y) + new_x;
        }
    }

    pub fn get_cursor_xy(&self) -> (usize, usize) {
        let cursor_line = self.text.char_to_line(self.cursor_pos);
        let line_start_char = self.text.line_to_char(cursor_line);
        let cursor_x = self.cursor_pos - line_start_char;
        let cursor_y = cursor_line;
        (cursor_x, cursor_y)
    }

    pub fn from_path(path: &str) -> io::Result<Self> {
        if !Path::new(path).exists() {
            File::create(path)?;
        }
        let text = Rope::from_reader(&mut BufReader::new(File::open(&path)?))?;

        Ok(Buffer {
            text,
            file_path: Some(PathBuf::from(path)),
            status: Status::Clean,
            cursor_pos: 0,
            file_props: Some(FileProps::new())
        })
    }

    pub fn get_line(&self, idx: usize) -> RopeSlice {
        self.text.line(idx)
    }

    pub fn bytes(&self) -> Bytes {
        self.text.bytes()
    }

    pub fn chars(&self) -> Chars {
        self.text.chars()
    }

    pub fn lines(&self) -> Lines {
        self.text.lines()
    }

    pub fn chunks(&self) -> Chunks {
        self.text.chunks()
    }

    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn save(&mut self) -> io::Result<String> {
        self.status = Status::Saving;
        let file_path = self.file_path.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "There's no associated file path with this buffer",
            )
        })?;
        let file = BufWriter::new(File::create(file_path)?);
        self.text.write_to(file)?;
        self.status = Status::Clean;
        Ok(format!("Wrote {} bytes to {}", self.text.len_bytes(), file_path.display()))
    }

    pub fn insert_char(&mut self, c: char) {
        self.text.insert_char(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.status = Status::Modified;
    }

    pub fn delete_char(&mut self) -> crossterm::Result<()> {
        if self.cursor_pos > 0 {
            if let Some(file_props) = &self.file_props {
                let ending: String = file_props.line_ending();
                if  ending.eq("\r\n") &&
                    self.cursor_pos > 2 &&
                    self.text.slice((self.cursor_pos - 2)..self.cursor_pos).eq("\r\n") {
                        self.text.remove((self.cursor_pos - 2)..self.cursor_pos);
                        self.cursor_pos -= 2;
                    } else {
                        self.text.remove((self.cursor_pos - 1)..self.cursor_pos);
                        self.cursor_pos -= 1;
                    }
            }
            // I don't know how efficient this is, but it fixes the issue where
            // when the user removes a bunch of new lines, it wouldn't refresh
            // what was underneath the cursor so there were "ghost" images
            // of the text that used to be there
            execute!(std::io::stdout(), terminal::Clear(ClearType::FromCursorDown))?;
            self.status = Status::Modified;
        }
        Ok(())
    }

    pub fn insert_newline(&mut self) -> crossterm::Result<()> {
        if let Some(file_props) = &self.file_props {
            let ending: String = file_props.line_ending();
            self.text.insert(self.cursor_pos, &ending);
            // How much to move to the right to be in front of the newline character(s).
            if ending.eq("\r\n") {
                self.cursor_pos += 2;
            } else {
                self.cursor_pos += 1;
            }
        }
        execute!(std::io::stdout(), terminal::Clear(ClearType::FromCursorDown))?;
        Ok(())
    }
}
