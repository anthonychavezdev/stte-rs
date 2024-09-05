use crossterm::event;
use crossterm::event::{Event, KeyEvent};
use std::time::Duration;

pub struct EventHandler;

impl EventHandler {
    pub fn get_events(&self) -> crossterm::Result<Event> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                return event::read();
            }
        }
    }
}
