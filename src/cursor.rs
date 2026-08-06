use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub const TAB_STOP: usize = 4;

#[derive(Default)]
pub struct Cursor {
    line: usize,
    col: usize,
    desired_col: usize
}

impl Cursor {
    pub fn col(&self) -> usize {
        self.col
    }

    pub fn set_col(&mut self, col: usize) {
        self.col = col
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn set_line(&mut self, line: usize) {
        self.line = line
    }

    pub fn desired_col(&self) -> usize {
        self.desired_col
    }

    pub fn set_desired_col(&mut self, desired_col: usize) {
        self.desired_col = desired_col;
    }
}


pub fn grapheme_width(grapheme: &str, col: usize) -> usize {
    if grapheme == "\t" {
        TAB_STOP - (col % TAB_STOP)
    } else {
        grapheme.width()
    }
}

pub fn display_width(text: &str) -> usize {
    let mut col = 0;
    for g in text.graphemes(true) {
        col += grapheme_width(g, col);
    }
    col
}

