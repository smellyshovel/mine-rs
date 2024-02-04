//! The terminal application updater.

use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use miners::MinesweeperError;

/// The support for the app.rs controls. Each app.rs variant must know what to do when something's being requested.
pub trait ControlsSupport {
    fn move_cursor(&mut self, direction: MoveCursorDirection);
    fn perform_main_action(&mut self) -> Result<(), MinesweeperError>;
    fn perform_secondary_action(&mut self) -> Result<(), MinesweeperError>;
    fn pause(&mut self);
    fn leave(&mut self, force: bool);
}

/// The available directions to move the cursor to.
#[derive(PartialEq)]
pub enum MoveCursorDirection {
    Up,
    Left,
    Down,
    Right,
}

pub fn update(app: &mut App, key_event: KeyEvent) -> Result<(), MinesweeperError> {
    use MoveCursorDirection::*;

    match key_event.code {
        KeyCode::Up | KeyCode::Char('i') | KeyCode::Char('w') => app.move_cursor(Up),
        KeyCode::Left | KeyCode::Char('j') | KeyCode::Char('a') => app.move_cursor(Left),
        KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('s') => app.move_cursor(Down),
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('d') => app.move_cursor(Right),
        KeyCode::Enter | KeyCode::Char(' ') => app.perform_main_action()?,
        KeyCode::Char('f') => app.perform_secondary_action()?,
        KeyCode::Char('p') => app.pause(),
        KeyCode::Esc | KeyCode::Char('q') => app.leave(false),
        KeyCode::Char('c') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.leave(true);
            }
        }
        _ => {}
    };

    Ok(())
}
