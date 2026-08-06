use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub const TAB_STOP: usize = 4;

pub enum Direction {
    Up,
    Right,
    Down,
    Left
}

#[derive(Default)]
pub struct Cursor {
    line_indx: usize,
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

    pub fn line_indx(&self) -> usize {
        self.line_indx
    }

    pub fn set_line_indx(&mut self, line: usize) {
        self.line_indx = line
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

pub fn prev_grapheme_boundary(line: &str, byte: usize) -> usize {
    line[..byte]
        .grapheme_indices(true)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0)
}

pub fn next_grapheme_boundary(line: &str, byte: usize) -> usize {
    line[byte..]
        .graphemes(true)
        .next()
        .map(|g| byte + g.len())
        .unwrap_or(line.len())
}

pub fn byte_at_display_col(line: &str, target: usize) -> usize {
    let mut col = 0;
    let mut byte = 0;
    for g in line.graphemes(true) {
        let w = grapheme_width(g, col);
        if col + w > target {
            break;
        }
        col += w;
        byte += g.len();
    }
    byte
}
