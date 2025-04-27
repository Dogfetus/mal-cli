use ratatui::Frame;
use crate::app::{App, Action};
use std::collections::HashMap;
use std::sync::OnceLock; 


mod launch;
mod info;
mod list;
mod profile;
mod settings;
mod overview;
mod widgets;
mod login;

static SCREENS: OnceLock<HashMap<String, Box<dyn Screen>>> = OnceLock::new();

// TODO: make these screens structs and implement the trait for them.
// TODO: they should take care of their own buttons and such
// when adding new screens add them in get_screen then just call get_screen when you need to draw them

#[allow(dead_code, unused_variables)]
pub trait Screen: Send + Sync {
    fn draw(&self, frame: &mut Frame, app: &App);
    fn handle_input(&mut self, key_event: crossterm::event::KeyEvent) -> Option<Action> {None}

    // TODO: you are tying to save the state of the screen and retreive it if it exists, 
    // right now the clone_box function is needed for save cloning even if only one
    // but this needs to be implemented for each screens, unless default is used, idk if that can
    // be done
    // here https://claude.ai/chat/51e961cd-bb71-4202-a1d7-3247fdd33666
    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {}
}

pub fn get_screen(screen_name: &str) -> Box<dyn Screen> {
    match screen_name {
        "Anime" => Box::new(info::InfoPage{}),
        "Manga" => Box::new(info::InfoPage{}),
        "Login" => Box::new(login::LoginPage::new()),
        _ => Box::new(launch::LaunchPage::new()),
    }
}

pub fn default() -> Box<dyn Screen> {
    get_screen("launch")
}
