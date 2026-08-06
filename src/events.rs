use std::{io, time::Duration};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, poll, read};

use crate::{cursor::Direction, display::Display};

pub enum Action {
    Quit,
    Redraw,
    Save,
    CursorMovement(Direction),
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
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        } => {
            Action::Save
        },
        KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE
        } => Action::CursorMovement(Direction::Up),
        KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE
        } => Action::CursorMovement(Direction::Right),
        KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE
        } => Action::CursorMovement(Direction::Down),
        KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE
        } => Action::CursorMovement(Direction::Left),
        _ => {
            Action::Idle
        }
    }
}

pub fn get_event_actions(display: &mut Display) -> io::Result<Action> {
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
