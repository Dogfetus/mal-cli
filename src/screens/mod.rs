use ratatui::Frame;
use ratatui_image::errors::Errors;
use crate::app::{Action, Event};
use crate::mal;
use crate::utils::customThreadProtocol::CustomResizeResponse;
use std::sync::{mpsc, Arc};
use std::collections::HashMap;
use std::thread::JoinHandle;
use std::any::Any;

#[allow(non_snake_case)]
mod screenTemplate;
mod settings;
mod overview;
mod profile;
mod widgets;
mod seasons;
mod search;
mod launch;
mod login;
mod info;
mod list;

// this is a macro to define screens in a more structured way
// it allows for screens to be implemented in a single place and work across the app
macro_rules! define_screens {

    // screens provided like bellow: 
    // SCREEN1 => "Screen1" => <module>::<structName>,
    ($($name:ident => $display:literal => $module:ident::$struct:ident),* $(,)?) => {
        // this is a module with a const of all available screens
        pub mod screens {
            $(
                pub const $name: &str = concat!($display, "Screen");
            )*
        }

        // this function gives a screen based on its name
        pub fn name_to_screen(screen_name: &str) -> &'static str {
            match screen_name {
                $(
                    $display => screens::$name,
                )*
                _ => screens::LAUNCH,
            }
        }

        // this function returns a display name for the screen
        pub fn screen_to_name(screen_name: &str) -> &str {
            match screen_name {
                $(
                    screens::$name => $display,
                )*
                _ => screen_name.strip_suffix("Screen").unwrap_or(screen_name),
            }
        }

        // this function creates a new screen based on its name
        pub fn create_screen(screen_name: &str) -> Box<dyn Screen> {
            match screen_name {
                $(
                    screens::$name => Box::new($module::$struct::new()),
                )*
                _ => Box::new(launch::LaunchScreen::new()),
            }
        }

    };
}








// INFO: make these screens structs and implement the trait for them.
// INFO: they should take care of their own buttons and such
// when adding new screens add them in define_screens then just call change_screen when you need to draw them
// INFO: Now the screen states are being stored in a hashmap, this could be changed to another
// structure (idk whats best, research this in the future, not important now)
// this could be moved to a screen manager
// but the main focus now is just implementing the screens and making them work
// and then connecting the login functionality to the app
// then use the rest of the mal api 
// then start inspecting tachyonfx
// INFO: here:
define_screens! {
    LAUNCH => "Launch" => launch::LaunchScreen,
    INFO => "Info" => info::InfoScreen,
    OVERVIEW => "Overview" => overview::OverviewScreen,
    SETTINGS => "Settings" => settings::SettingsScreen,
    LOGIN => "Login" => login::LoginScreen,
    PROFILE => "Profile" => profile::ProfileScreen,
    SEASONS => "Seasons" => seasons::SeasonsScreen,
    SEARCH => "Search" => search::SearchScreen,
    LIST => "List" => list::ListScreen, 

    // Add more as needed:
    // SCREEN1 => "Screen1" => <module>::<structName>,
    // SCREEN2 => "Screen2" => <module>::<structName>,
    // etc...
}









#[derive(Debug, Clone)]
pub struct BackgroundInfo {
    // pub stop: Arc<AtomicBool>,
    pub app_sx: mpsc::Sender<Event>,
    pub mal_client: Arc<mal::MalClient>,
}

#[allow(dead_code, unused_variables)]
pub trait Screen: Send{
    fn draw(&self, frame: &mut Frame);
    fn handle_input(&mut self, key_event: crossterm::event::KeyEvent) -> Option<Action> {None}
    fn get_name(&self) -> String {
        let name = std::any::type_name::<Self>();
        name.split("::").last().unwrap_or(name).to_string()
    }
    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        panic!("Attempted to clone a screen type that doesn't support cloning: {}", 
               self.get_name());
    }
    fn should_store(&self) -> bool { true }

    //INFO: just create a backgground function that returns a JoinHandle and the screen will have
    //background functionality. Use apply update to pass updates to the rendering thread
    fn background(&mut self, info: BackgroundInfo) -> Option<JoinHandle<()>> {
        None
    }
    fn apply_update(&mut self, update: BackgroundUpdate) {
    }
    fn image_redraw(&mut self, id: usize, response: Result<CustomResizeResponse, Errors>) {
        // Default implementation does nothing
    }
}


pub struct ScreenManager {
    current_screen: Box<dyn Screen>,
    screen_storage: HashMap<String, Box<dyn Screen>>,
    backgrounds: Vec<JoinHandle<()>>,
    passable_info: BackgroundInfo,
}

#[allow(dead_code)]
impl ScreenManager {
    pub fn new(app_sx: mpsc::Sender<Event>, mal_client: Arc<mal::MalClient>) -> Self {
        Self {
            // default screen is the launch screen
            current_screen: Box::new(launch::LaunchScreen::new()),
            screen_storage: HashMap::new(),
            backgrounds: Vec::new(),
            passable_info: BackgroundInfo {
                // stop: Arc::new(AtomicBool::new(false)),
                app_sx,
                mal_client,
            },
        }
    }

    pub fn render_screen(&self, frame: &mut Frame) {
        self.current_screen.draw(frame);
    }

    pub fn handle_input(&mut self, key_event: crossterm::event::KeyEvent) -> Option<Action> {
        self.current_screen.handle_input(key_event)
    }

    pub fn update_screen(&mut self, update: BackgroundUpdate) {
        if self.current_screen.get_name() == update.screen_id
        {
            self.current_screen.apply_update(update);
        } else {
            // TODO: check if this actually works when not rendered?
            if let Some(screen) = self.screen_storage.get_mut(&update.screen_id) {
                screen.apply_update(update);
            }
        }
    }

    pub fn redraw_image(&mut self, id: usize, response: Result<CustomResizeResponse, Errors>) {
        self.current_screen.image_redraw(id, response);
    }

    // change screen stores the previous screen if not specified otherwise
    // the current screen is removed from the storage if it exists, or created anew
    // this allows for screens to be swapped and their state to be preserved
    pub fn change_screen(&mut self, screen_name: &str) {
        if self.current_screen.should_store() {
            self.screen_storage.insert(
                self.current_screen.get_name(),
                self.current_screen.clone_box()
            );
        }

        if let Some(screen) = self.screen_storage.remove(screen_name) {
            self.current_screen = screen;
        } else {
            self.current_screen = create_screen(screen_name);
        }

        self.cleanup_backgrounds();
        self.spawn_background();
    }

    pub fn spawn_background(&mut self) {
        if let Some(handle) = self.current_screen.background(self.passable_info.clone()) { 
            self.backgrounds.push(handle);
        }
    }

    // this stops all background threads and waits for them to finish
    pub fn stop_background(&mut self) {
        for handle in self.backgrounds.drain(..) {
            handle.join().unwrap();
        }
    }

    // this cleans up the backgrounds by removing those that are finished
    pub fn cleanup_backgrounds(&mut self) {
        self.backgrounds.retain(|handle| !handle.is_finished());
    }
}



#[derive(Debug)]
pub struct BackgroundUpdate {
    pub screen_id: String,
    pub updates: HashMap<String, Box<dyn Any + Send + Sync>>,
}

#[allow(dead_code)]
impl BackgroundUpdate {
    pub fn new(screen_id: String) -> Self {
        Self {
            screen_id,
            updates: HashMap::new(),
        }
    }

    pub fn set<T: Any + Send + Sync>(mut self, field: &str, value: T) -> Self {
        self.updates.insert(field.to_string(), Box::new(value));
        self
    }

    pub fn get<T: Any>(&self, field: &str) -> Option<&T> {
        self.updates.get(field)?.downcast_ref::<T>()
    }

    pub fn has(&self, field: &str) -> bool {
        self.updates.contains_key(field)
    }

    pub fn fields(&self) -> impl Iterator<Item = &String> {
        self.updates.keys()
    }

    pub fn take<T: Any + Send + Sync>(&mut self, field: &str) -> Option<T> {
        self.updates.remove(field)?
            .downcast::<T>()
            .ok()
            .map(|boxed| *boxed)
    }
}
