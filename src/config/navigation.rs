use serde::{Deserialize, Serialize};
use crossterm::event::KeyCode;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Navigation {
    pub nav_up: Vec<KeyCode>,
    pub nav_down: Vec<KeyCode>,
    pub nav_left: Vec<KeyCode>,
    pub nav_right: Vec<KeyCode>,
}

impl Default for Navigation {
    fn default() -> Self {
        Self {
            nav_up: vec![KeyCode::Up, KeyCode::Char('k')],
            nav_down: vec![KeyCode::Down, KeyCode::Char('j')],
            nav_left: vec![KeyCode::Left, KeyCode::Char('h')],
            nav_right: vec![KeyCode::Right, KeyCode::Char('l')],
        }
    }
}
