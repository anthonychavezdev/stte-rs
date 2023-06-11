use crossterm::terminal::ClearType;
use crossterm::{execute, terminal};
use ropey::iter::{Bytes, Chars, Chunks, Lines};
use ropey::{Rope, RopeSlice};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, ErrorKind};
use std::path::{PathBuf, Path};

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

#[derive(Debug)]
pub struct Buffer {
    text: Rope,                 // text from a file or in memory
    file_path: Option<PathBuf>, // path associated with a file. Buffers don't always need to be associated with a file, they can be in memory only
    status: Status, // Whether the buffer has been modified, left unchanged, or is being saved back to disk?
    cursor_pos: usize,
    line_endings: String
}

impl Buffer {
    pub fn new(path: Option<PathBuf>) -> Buffer {
        let text = Rope::new();
        Buffer {
            text,
            file_path: path,
            status: Status::Clean,
            cursor_pos: 0,
            line_endings: if cfg!(target_os = "windows") {
                "\r\n".to_string()
            } else {
                "\n".to_string()
            }
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

    /** The ropey cursor and the curosr that's actually shown in the editor
     are different cursors.
    This returns the width for characters so the cursors can be synced*/
    pub fn get_visual_char_len(&self) -> usize {
        let (cursor_x, cursor_y) = self.get_cursor_xy();
        let line = self.text.line(cursor_y);
        let mut visual_len: usize = 0;
        for ch in line.chars().take(cursor_x) {
            visual_len += match ch {
                '\t' => 8,
                _ => unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1)
            };
        }
        visual_len
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
                    line_endings: if cfg!(target_os = "windows") {
                        "\r\n".to_string()
                    } else {
                        "\n".to_string()
                    }
                })
            },
            Err(e) => {
                if e.kind() == ErrorKind::PermissionDenied {
                    Err(BufferError {
                        message: "Can't read file".to_string(),
                        cause: Some(e)
                    })
                } else if e.kind() == ErrorKind::NotFound {
                    Ok(Buffer {
                        text: Rope::new(),
                        file_path: Some(PathBuf::from(path)),
                        status: Status::Clean,
                        cursor_pos: 0,
                        line_endings: if cfg!(target_os = "windows") {
                            "\r\n".to_string()
                        } else {
                            "\n".to_string()
                        }
                    })
                } else {
                    Err(BufferError {
                        message: "Can't open file".to_string(),
                        cause: Some(e)
                    })
                }
            },
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
                        Ok(format!("Wrote {} bytes to {}", self.text.len_bytes(), path.display()))
                    },
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
                    },
                }
            }
            None => Err(BufferError {
                message: "No file associated with buffer".to_string(),
                cause: None,
            })
        }
    }


    pub fn insert_char(&mut self, c: char) {
        self.text.insert_char(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.status = Status::Modified;
    }

    pub fn delete_char(&mut self) -> crossterm::Result<()> {
        if self.cursor_pos > 0 {
            if self.line_endings.eq("\r\n") &&
                self.cursor_pos > 2 &&
                self.text.slice((self.cursor_pos - 2)..self.cursor_pos).eq("\r\n") {
                    self.text.remove((self.cursor_pos - 2)..self.cursor_pos);
                    self.cursor_pos -= 2;
                } else {
                    self.text.remove((self.cursor_pos - 1)..self.cursor_pos);
                    self.cursor_pos -= 1;
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
        self.text.insert(self.cursor_pos, &self.line_endings);
        // How much to move to the right to be in front of the newline character(s).
        if self.line_endings.eq("\r\n") {
            self.cursor_pos += 2;
        } else {
            self.cursor_pos += 1;
        }
        execute!(std::io::stdout(), terminal::Clear(ClearType::FromCursorDown))?;
        Ok(())
    }
}
