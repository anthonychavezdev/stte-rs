use std::{io, time::Duration};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, poll, read};

pub enum Action {
    Quit,
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

pub fn handle_events() -> io::Result<Action> {
    if !poll(Duration::from_millis(500))? {
        return Ok(Action::Idle);
    }
    let action = match read()? {
        Event::Key(key) => handle_key_press(key),
        _ => Action::Idle
    };

    Ok(action)
}
