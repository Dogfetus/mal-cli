use ratatui::Frame;
use screens::*;
use crate::app::{App, Action};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::HashMap;

mod launch;
mod info;
mod list;
mod profile;
mod settings;
mod overview;
mod widgets;
mod login;

// TODO: Now the screen states are being stored in a hashmap, this could be changed to another
// structure (idk whats best, research this in the future, not important now)
// this could be moved to a screen manager
// but the main focus now is just implementing the screens and making them work
// and then connecting the login functionality to the app
// then use the rest of the mal api 
// then start inspecting tachyonfx


static STORAGE : Lazy<Mutex<HashMap<String, Box<dyn Screen>>>> = Lazy::new(|| Mutex::new(HashMap::new()));


#[allow(dead_code)]
pub mod screens {
    pub const LAUNCH: &str = "LaunchScreen";
    pub const INFO: &str = "InfoScreen";
    pub const LIST: &str = "ListScreen";
    pub const PROFILE: &str = "ProfileScreen";
    pub const SETTINGS: &str = "SettingsScreen";
    pub const OVERVIEW: &str = "OverviewScreen";
    pub const LOGIN: &str = "LoginScreen";
    pub const ANIME: &str = "AnimeScreen";
    pub const MANGA: &str = "MangaScreen";
    pub const SEARCH: &str = "SearchScreen";
    pub const BROWSE: &str = "BrowseScreen";
}



// TODO: make these screens structs and implement the trait for them.
// TODO: they should take care of their own buttons and such
// when adding new screens add them in change_screen (and screens consts) then just call change_screen when you need to draw them
#[allow(dead_code, unused_variables)]
pub trait Screen: Send + Sync {
    fn draw(&self, frame: &mut Frame);
    fn handle_input(&mut self, key_event: crossterm::event::KeyEvent) -> Option<Action> {None}
    fn clone_box(&self) -> Box<dyn Screen + Send + Sync>;
    fn should_store(&self) -> bool { true }
    fn get_name(&self) -> String {
        let name = std::any::type_name::<Self>();
        name.split("::").last().unwrap_or(name).to_string()
    }
}


pub fn change_screen(app: &mut App, screen_name: &str){
    if app.current_screen.should_store() {
        store_screen(app.current_screen.clone_box());
    }

    if let Some(screen) = retreive_screen(screen_name) {
        app.current_screen = screen;
        return;
    }

    app.current_screen = match screen_name {
        INFO => Box::new(info::InfoScreen::new()),
        OVERVIEW => Box::new(overview::OverviewScreen::new()),
        SETTINGS => Box::new(settings::SettingsScreen::new()),
        LOGIN => Box::new(login::LoginScreen::new()),
        PROFILE => Box::new(profile::ProfileScreen::new()),
        _ => Box::new(launch::LaunchScreen::new()),
    };
}

pub fn default() -> Box<dyn Screen> {
    Box::new(launch::LaunchScreen::new())
}

pub fn store_screen(screen: Box<dyn Screen>) {
    let mut storage = STORAGE.lock().unwrap();
    storage.insert(screen.get_name(), screen.clone_box());
}

pub fn retreive_screen(screen_name: &str) -> Option<Box<dyn Screen>> {
    let storage = STORAGE.lock().unwrap();
    if let Some(screen) = storage.get(screen_name) {
        return Some(screen.clone_box());
    }
    None
}
