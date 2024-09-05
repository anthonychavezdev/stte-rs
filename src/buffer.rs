use crossterm::terminal::ClearType;
use crossterm::{execute, terminal};
use ropey::iter::{Bytes, Chars, Chunks, Lines};
use ropey::{Rope, RopeSlice};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, ErrorKind};
use std::path::{Path, PathBuf};
use unicode_width::UnicodeWidthChar;

use crate::screen::Screen;

const TAB_WIDTH: usize = 8;

#[derive(Debug)]
pub struct BufferError {
    message: String,
    cause: Option<io::Error>,
}

impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(cause) = &self.cause {
            write!(f, ": {}", cause)?;
        }
        Ok(())
    }
}

impl Error for BufferError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().map(|e| e as &(dyn Error + 'static))
    }
}

impl From<io::Error> for BufferError {
    fn from(error: io::Error) -> Self {
        BufferError {
            message: "I/O error occurred".to_string(),
            cause: Some(error),
        }
    }
}

#[derive(Debug)]
pub enum Status {
    Modified,
    Clean,
    Saving,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineEnding {
    LF,
    CRLF,
}

impl LineEnding {
    fn as_str(&self) -> &'static str {
        match self {
            LineEnding::LF => "\n",
            LineEnding::CRLF => "\r\n",
        }
    }

    fn len(&self) -> usize {
        match self {
            LineEnding::LF => 1,
            LineEnding::CRLF => 2,
        }
    }
}

#[derive(Debug)]
pub struct Buffer {
    text: Rope,                 // text from a file or in memory
    file_path: Option<PathBuf>, // path associated with a file. Buffers don't always need to be associated with a file, they can be in memory only
    status: Status, // Whether the buffer has been modified, left unchanged, or is being saved back to disk?
    cursor_pos: usize,
    line_ending: LineEnding,
}

impl Buffer {
    pub fn new(path: Option<PathBuf>) -> Buffer {
        let text = Rope::new();
        Buffer {
            text,
            file_path: path,
            status: Status::Clean,
            cursor_pos: 0,
            line_ending: if cfg!(target_os = "windows") {
                LineEnding::CRLF
            } else {
                LineEnding::LF
            },
        }
    }

    pub fn cursor_row(&self) -> usize {
        self.text.char_to_line(self.cursor_pos)
    }

    pub fn cursor_column(&self) -> usize {
        let line_start = self.text.line_to_char(self.cursor_row());
        self.cursor_pos - line_start
    }

    /** The ropey cursor and the curosr that's actually shown in the editor
    are different cursors.
    This returns the width for characters so the cursors can be synced*/
    pub fn get_char_column_width(&self, x: usize, y: usize) -> usize {
        let mut visual_width = 0;
        for (idx, ch) in self.text.line(y).chars().take(x).enumerate() {
            visual_width += match ch {
                '\t' => TAB_WIDTH - (visual_width % TAB_WIDTH),
                _ => ch.width().unwrap_or(1),
            };
        }
        visual_width
    }

    pub fn get_visual_cursor_x(&self) -> usize {
        let (cursor_x, cursor_y) = self.get_cursor_xy();
        self.get_char_column_width(cursor_x, cursor_y)
    }

    fn get_char_index_from_visual_x(&self, line: usize, target_visual_x: usize) -> usize {
        let mut visual_x = 0;
        for (idx, ch) in self.text.line(line).chars().enumerate() {
            let char_width = match ch {
                '\t' => TAB_WIDTH - (visual_x % TAB_WIDTH),
                _ => ch.width().unwrap_or(1),
            };
            if visual_x + char_width > target_visual_x {
                return idx;
            }
            visual_x += char_width;
        }
        self.text.line(line).len_chars()
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
        let (cursor_x, cursor_y) = self.get_cursor_xy();
        if cursor_y > 0 {
            let target_y = cursor_y - 1;
            let target_line = self.text.line(target_y);
            let visual_x = self.get_char_column_width(cursor_x, cursor_y);
            let new_x = self.get_char_index_from_visual_x(target_y, visual_x);
            self.cursor_pos = self.text.line_to_char(target_y) + new_x;
        }
    }

    pub fn move_cursor_down(&mut self) {
        let (cursor_x, cursor_y) = self.get_cursor_xy();
        if cursor_y < self.text.len_lines() - 1 {
            let target_y = cursor_y + 1;
            let target_line = self.text.line(target_y);
            let visual_x = self.get_char_column_width(cursor_x, cursor_y);
            let new_x = self.get_char_index_from_visual_x(target_y, visual_x);
            self.cursor_pos = self.text.line_to_char(target_y) + new_x;
        }
    }
    pub fn get_cursor_xy(&self) -> (usize, usize) {
        let line_idx = self.text.char_to_line(self.cursor_pos);
        let line_start = self.text.line_to_char(line_idx);
        (self.cursor_pos - line_start, line_idx)
    }
    pub fn from_path(path: &str) -> Result<Self, BufferError> {
        let path = Path::new(path);
        let file = File::open(&path);

        match file {
            Ok(file) => {
                let text = Rope::from_reader(&mut BufReader::new(file))?;
                Ok(Buffer {
                    text,
                    file_path: Some(PathBuf::from(path)),
                    status: Status::Clean,
                    cursor_pos: 0,
                    line_ending: if cfg!(target_os = "windows") {
                        LineEnding::CRLF
                    } else {
                        LineEnding::LF
                    },
                })
            }
            Err(e) => {
                if e.kind() == ErrorKind::PermissionDenied {
                    Err(BufferError {
                        message: "Can't read file".to_string(),
                        cause: Some(e),
                    })
                } else if e.kind() == ErrorKind::NotFound {
                    Ok(Buffer {
                        text: Rope::new(),
                        file_path: Some(PathBuf::from(path)),
                        status: Status::Clean,
                        cursor_pos: 0,
                        line_ending: if cfg!(target_os = "windows") {
                            LineEnding::CRLF
                        } else {
                            LineEnding::LF
                        },
                    })
                } else {
                    Err(BufferError {
                        message: "Can't open file".to_string(),
                        cause: Some(e),
                    })
                }
            }
        }
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

    pub fn save(&mut self) -> Result<String, BufferError> {
        self.status = Status::Saving;
        match &self.file_path {
            Some(path) => {
                let file = File::create(&path);
                match file {
                    Ok(mut file) => {
                        self.text.write_to(&mut file)?;
                        self.status = Status::Clean;
                        Ok(format!(
                            "Wrote {} bytes to {}",
                            self.text.len_bytes(),
                            path.display()
                        ))
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::PermissionDenied {
                            Err(BufferError {
                                message: "Can't write to file".to_string(),
                                cause: Some(e),
                            })
                        } else {
                            Err(BufferError {
                                message: "I/O error occurred".to_string(),
                                cause: Some(e),
                            })
                        }
                    }
                }
            }
            None => Err(BufferError {
                message: "No file associated with buffer".to_string(),
                cause: None,
            }),
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.text.insert_char(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.status = Status::Modified;
    }

    pub fn delete_char(&mut self) -> crossterm::Result<()> {
        if self.cursor_pos > 0 {
            let start = self.cursor_pos.saturating_sub(self.line_ending.len());
            if self.text.slice(start..self.cursor_pos) == self.line_ending.as_str() {
                self.text.remove(start..self.cursor_pos);
                self.cursor_pos = start;
            } else {
                self.text.remove((self.cursor_pos - 1)..self.cursor_pos);
                self.cursor_pos -= 1;
            }
            // I don't know how efficient this is, but it fixes the issue where
            // when the user removes a bunch of new lines, it wouldn't refresh
            // what was underneath the cursor so there were "ghost" images
            // of the text that used to be there
            execute!(
                std::io::stdout(),
                terminal::Clear(ClearType::FromCursorDown)
            )?;
            self.status = Status::Modified;
        }
        Ok(())
    }

    pub fn insert_newline(&mut self) -> crossterm::Result<()> {
        self.text.insert(self.cursor_pos, self.line_ending.as_str());
        // How much to move to the right to be in front of the newline character(s).
        self.cursor_pos += self.line_ending.len();
        execute!(
            std::io::stdout(),
            terminal::Clear(ClearType::FromCursorDown)
        )?;
        Ok(())
    }
}
