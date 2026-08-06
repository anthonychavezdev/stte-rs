use std::{io, time::Duration};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, poll, read};

use crate::display::Display;

pub enum Action {
    Quit,
    Redraw,
    Idle
}

fn handle_key_press(key: KeyEvent) -> Action {
    match key {
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE
        } => {
            Action::Quit
        },
        _ => {
            Action::Idle
        }
    }
}

pub fn handle_events(display: &mut Display) -> io::Result<Action> {
    if !poll(Duration::from_millis(500))? {
        return Ok(Action::Idle);
    }
    let action = match read()? {
        Event::Key(key) => handle_key_press(key),
        Event::Resize(width, height) => {
            display.set_dimensions(width, height);
            Action::Redraw
        }
        _ => Action::Idle
    };

    Ok(action)
}
