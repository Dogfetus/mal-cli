use ratatui::Frame;
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


#[allow(dead_code, unused_variables)]
static STORAGE : Lazy<Mutex<HashMap<String, Box<dyn Screen>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// TODO: make these screens structs and implement the trait for them.
// TODO: they should take care of their own buttons and such
// when adding new screens add them in get_screen then just call get_screen when you need to draw them




// TODO: https://claude.ai/chat/6ec6a60b-05ce-4d9a-9b22-7ab4e06d0cc9
// TODO: current issue is that this works, but it saves the new verison of the screen
// and does not update the saved version when changes happen (switches screens)
// fix this
#[allow(dead_code, unused_variables)]
pub trait Screen: Send + Sync {
    fn draw(&self, frame: &mut Frame, app: &App);
    fn handle_input(&mut self, key_event: crossterm::event::KeyEvent) -> Option<Action> {None}
    fn clone_box(&self) -> Box<dyn Screen + Send + Sync>;
    fn should_store(&self) -> bool { true }
}


pub fn get_screen(screen_name: &str) -> Box<dyn Screen> {
    {
        let storage = STORAGE.lock().unwrap();
        if let Some(screen) = storage.get(screen_name) {
            return screen.clone_box();
        }
    }

    println!("Creating new screen: {}", screen_name);

    let screen: Box<dyn Screen> = match screen_name {
        "Anime" => Box::new(info::InfoPage{}),
        "Manga" => Box::new(info::InfoPage{}),
        "Login" => Box::new(login::LoginPage::new()),
        _ => Box::new(launch::LaunchPage::new()),
    };

    if screen.should_store() {
        store_screen(screen_name, screen.clone_box());
    }

    screen

}

pub fn default() -> Box<dyn Screen> {
    get_screen("launch")
}

pub fn store_screen(name: &str, screen: Box<dyn Screen>) {
    if screen.should_store() {
        let mut storage = STORAGE.lock().unwrap();
        storage.insert(name.to_string(), screen.clone_box());
    }
}
