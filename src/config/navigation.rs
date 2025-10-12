use serde::{Deserialize, Serialize};
use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDirection {
    Up,
    Down,
    Left,
    Right,
    None,
}

fn def_up() -> Vec<KeyCode> {
    vec![KeyCode::Up, KeyCode::Char('k')]
}

fn def_down() -> Vec<KeyCode> {
    vec![KeyCode::Down, KeyCode::Char('j')]
}

fn def_left() -> Vec<KeyCode> {
    vec![KeyCode::Left, KeyCode::Char('h')]
}

fn def_right() -> Vec<KeyCode> {
    vec![KeyCode::Right, KeyCode::Char('l')]
}

fn def_select() -> Vec<KeyCode> {
    vec![KeyCode::Enter, KeyCode::Char(' ')]
}

fn def_close() -> Vec<KeyCode> {
    vec![KeyCode::Esc, KeyCode::Char('q')]
}

fn def_mouse_capture() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Navigation {

    // basic navigation in the application
    #[serde(default = "def_up")]
    pub nav_up: Vec<KeyCode>,

    #[serde(default = "def_down")]
    pub nav_down: Vec<KeyCode>,

    #[serde(default = "def_left")]
    pub nav_left: Vec<KeyCode>,

    #[serde(default = "def_right")]
    pub nav_right: Vec<KeyCode>,

    #[serde(default = "def_select")]
    pub select: Vec<KeyCode>,

    #[serde(default = "def_close")]
    pub close: Vec<KeyCode>,

    // enable mouse capture in the terimnal for mouse navigation 
    #[serde(default = "def_mouse_capture")]
    pub enable_mouse_capture: bool,
}

impl Default for Navigation {
    fn default() -> Self {
        Self {
            nav_up: def_up(),
            nav_down: def_down(),
            nav_left: def_left(),
            nav_right: def_right(),
            select: def_select(),
            close: def_close(),
            enable_mouse_capture: def_mouse_capture(),
        }
    }
}
impl Navigation {
    // get the navigation direction based on the key pressed
    pub fn get_direction(&self, key: &KeyCode) -> NavDirection {
        if self.nav_up.contains(key) {
            NavDirection::Up
        } else if self.nav_down.contains(key) {
            NavDirection::Down
        } else if self.nav_left.contains(key) {
            NavDirection::Left
        } else if self.nav_right.contains(key) {
            NavDirection::Right
        } else {
            NavDirection::None 
        }
    }

    // if the select key is pressed
    pub fn is_select(&self, key: &KeyCode) -> bool {
        self.select.contains(key)
    }

    // if the close key is pressed
    pub fn is_close(&self, key: &KeyCode) -> bool {
        self.close.contains(key)
    }
}
