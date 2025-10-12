use serde::{Deserialize, Serialize};
use crossterm::event::KeyCode;

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
            enable_mouse_capture: def_mouse_capture(),
        }
    }
}
