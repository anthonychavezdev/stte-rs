use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, terminal};
use std::io::{stdout, Write};

pub struct Screen {
    win_size: (u16, u16),
}

impl Screen {
    pub fn new() -> Self {
        let win_size = terminal::size().map(|(x, y)| (x as u16, y as u16)).unwrap();
        Self { win_size }
    }

    fn draw_rows(&self) {
        let screen_rows = self.win_size.1;
        for i in 0..screen_rows {
            print!("~");
            if i < screen_rows - 1 {
                println!("\r");
            }
            if let Ok(()) = stdout().flush() {}
        }
    }

    pub fn clear() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    pub fn refresh(&self) -> crossterm::Result<()> {
        Self::clear()?;
        self.draw_rows();
        execute!(stdout(), cursor::MoveTo(0, 0))
    }
}
