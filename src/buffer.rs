use ropey::iter::{Bytes, Chars, Chunks, Lines};
use ropey::{Rope, RopeSlice};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;

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
    cursor_pos: i32,
}

impl Buffer {
    pub fn new() -> Buffer {
        let text = Rope::new();
        Buffer {
            text,
            file_path: None,
            status: Status::Clean,
            cursor_pos: 0
        }
    }

    pub fn from_path(path: &str) -> io::Result<Self> {
        let text = Rope::from_reader(&mut BufReader::new(File::open(&path)?))?;
        Ok(Buffer {
            text,
            file_path: Some(PathBuf::from(path)),
            status: Status::Clean,
            cursor_pos: 0,
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

    pub fn save(&mut self) -> io::Result<()> {
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
        Ok(())
    }

    pub fn edit(&mut self, start: usize, end: usize, text: &str) {
        if start != end {
            self.text.remove(start..end);
        }
        if !text.is_empty() {
            self.text.insert(start, text);
        }
        self.status = Status::Modified;
    }
}
