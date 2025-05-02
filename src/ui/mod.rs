use ratatui::Frame;
use screens::*;
use crate::app::{Action, Event};
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};
use std::collections::HashMap;
use std::thread::{self, JoinHandle};

mod launch;
mod info;
mod list;
mod profile;
mod settings;
mod overview;
mod widgets;
mod login;

// INFO: make these screens structs and implement the trait for them.
// INFO: they should take care of their own buttons and such
// when adding new screens add them in change_screen (and screens consts) then just call change_screen when you need to draw them
// INFO: Now the screen states are being stored in a hashmap, this could be changed to another
// structure (idk whats best, research this in the future, not important now)
// this could be moved to a screen manager
// but the main focus now is just implementing the screens and making them work
// and then connecting the login functionality to the app
// then use the rest of the mal api 
// then start inspecting tachyonfx

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


//TODO: add passable background logic to each screen that can be passed to a background process
//TODO: after gathering thoughts, the screen should spawn its background upon creation / screenswap
//TODO: the background should be passed a channel to send events to the rendering thread 
//TODO: the background process will be updated by the screen running (currenlty handle_input) notify background, or something

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
    fn background(&self, sx: mpsc::Sender<Event>, stop: Arc<AtomicBool>) -> Option<JoinHandle<()>> {
        None
    }
}


pub struct ScreenManager {
    current_screen: Box<dyn Screen>,
    screen_storage: HashMap<String, Box<dyn Screen>>,
    backgrounds: Vec<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    sx: mpsc::Sender<Event>,
}

#[allow(dead_code)]
impl ScreenManager {
    pub fn new(sx: mpsc::Sender<Event>) -> Self {
        Self {
            current_screen: Box::new(launch::LaunchScreen::new()),
            screen_storage: HashMap::new(),
            backgrounds: Vec::new(),
            stop: Arc::new(AtomicBool::new(false)),
            sx,
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        self.current_screen.draw(frame);
    }

    pub fn handle_input(&mut self, key_event: crossterm::event::KeyEvent) -> Option<Action> {
        self.current_screen.handle_input(key_event)
    }

    pub fn change_screen(&mut self, screen_name: &str) {
        if self.current_screen.should_store() {
            self.screen_storage.insert(
                self.current_screen.get_name(),
                self.current_screen.clone_box()
            );
        }

        if let Some(screen) = self.screen_storage.get(screen_name) {
            self.current_screen = screen.clone_box();
        } else {
            self.current_screen = match screen_name {
                INFO => Box::new(info::InfoScreen::new()),
                OVERVIEW => Box::new(overview::OverviewScreen::new()),
                SETTINGS => Box::new(settings::SettingsScreen::new()),
                LOGIN => Box::new(login::LoginScreen::new()),
                PROFILE => Box::new(profile::ProfileScreen::new()),
                _ => Box::new(launch::LaunchScreen::new()),
            };
        }

        self.cleanup_backgrounds();
        self.spawn_background();
    }

    pub fn spawn_background(&mut self) {
        if let Some(handle) = self.current_screen.background(self.sx.clone(), self.stop.clone()) { 
            self.backgrounds.push(handle);
        }
    }

    pub fn stop_background(&mut self) {
        for handle in self.backgrounds.drain(..) {
            handle.join().unwrap();
        }
    }

    pub fn cleanup_backgrounds(&mut self) {
        self.backgrounds.retain(|handle| !handle.is_finished());
    }
}
