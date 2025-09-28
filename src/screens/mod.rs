use crate::app::{Action, Event, ExtraInfo};
use crate::mal::models::anime::AnimeId;
use crate::send_error;
use ratatui::Frame;
use ratatui::layout::Layout;
use screens::*;
use std::any::Any;
use std::collections::HashMap;
use std::thread::JoinHandle;
use widgets::{navbar, popup};

#[allow(non_snake_case)]
mod screenTemplate;
mod settings;
mod overview;
mod seasons;
mod widgets;
mod profile;
mod launch;
mod search;
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
        pub fn create_screen(screen_name: &str, app_info: &ExtraInfo) -> Box<dyn Screen> {
            match screen_name {
                $(
                    screens::$name => Box::new($module::$struct::new(app_info.clone())),
                )*
                _ => Box::new(launch::LaunchScreen::new(app_info.clone())),
            }
        }
    };
}


#[macro_export]
macro_rules! add_screen_caching {
    () => {
        fn should_store(&self) -> bool {
            true
        }

        fn clone_box(&self) -> Box<dyn Screen> {
            Box::new(self.clone())
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
// INFO: Variable => Display name => Screen struct
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

    // To add more::
    // SCREEN1 => "<structName>" => <module>::<structName>Screen,
    // SCREEN2 => "<structName>" => <module>::<structName>Screen,
    // etc...
}


#[allow(dead_code, unused_variables)]
pub trait Screen {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_keyboard(&mut self, key_event: crossterm::event::KeyEvent) -> Option<Action> {
        None
    }
    fn handle_mouse(&mut self, mouse_event: crossterm::event::MouseEvent) -> Option<Action> {
        None
    }
    fn get_name(&self) -> String {
        let name = std::any::type_name::<Self>();
        name.split("::").last().unwrap_or(name).to_string()
    }
    fn clone_box(&self) -> Box<dyn Screen> {
        panic!(
            "Attempted to clone a screen type that doesn't support cloning: {}",
            self.get_name()
        );
    }
    fn should_store(&self) -> bool {
       false 
    }
    fn uses_navbar(&self) -> bool {
        true
    } 

    //INFO: just create a backgground function that returns a JoinHandle and the screen will have
    //background functionality. Use apply update to pass updates to the rendering thread
    fn background(&mut self) -> Option<JoinHandle<()>> {
        None
    }
    fn apply_update(&mut self, update: BackgroundUpdate) {}
}

pub struct ScreenManager {
    navbar: navbar::NavBar,
    overlay: popup::AnimePopup,
    error_overlay: popup::ErrorPopup,
    current_screen: Box<dyn Screen>,
    screen_storage: HashMap<String, Box<dyn Screen>>,
    backgrounds: Vec<JoinHandle<()>>,
    passable_info: ExtraInfo,
}

#[allow(dead_code)]
impl ScreenManager {
    pub fn new(passable_info: ExtraInfo) -> Self {

        Self {
            // default screen is the launch screen
            navbar: navbar::NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(SEARCH)
                .add_screen(LIST)
                .add_screen(PROFILE),
            overlay: popup::AnimePopup::new(passable_info.clone()),
            error_overlay: popup::ErrorPopup::new(),
            current_screen: Box::new(launch::LaunchScreen::new(passable_info.clone())),
            screen_storage: HashMap::new(),
            backgrounds: Vec::new(),
            passable_info,
        }
    }

    pub fn render_screen(&mut self, frame: &mut Frame) {
        self.current_screen.draw(frame);
        if self.current_screen.uses_navbar() {
            let nav_bar_area = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(3),
                    ratatui::layout::Constraint::Fill(1),
                ])
                .split(frame.area())[0];
            self.navbar.render(frame, nav_bar_area);
        }
        self.overlay.render(frame);
        self.error_overlay.render(frame);
    }

    pub fn toggle_navbar(&mut self, select: bool) {
        if select {
            self.navbar.select();
        } else {
            self.navbar.deselect();
        }
    }

    pub fn toggle_overlay(&mut self, anime: AnimeId) {
        self.overlay.set_anime(anime);
        self.overlay.open();
    }

    pub fn refresh(&mut self) {
        self.overlay.update_buttons();
    }

    pub fn show_error(&mut self, error: String) {
        self.error_overlay.set_error(error);
        self.error_overlay.open();
    }

    pub fn handle_input(&mut self, event: crossterm::event::Event) -> Option<Action> {
        match event {
            crossterm::event::Event::Key(key_event) => {
                if self.error_overlay.is_open() {
                    return self.error_overlay.handle_keyboard(key_event);
                }

                if self.overlay.is_open() {
                    return self.overlay.handle_keyboard(key_event);
                }

                if self.navbar.is_selected() {
                    return self.navbar.handle_keyboard(key_event)
                        .and_then(|action| match action {
                        Action::NavbarSelect(_) => self.current_screen.handle_keyboard(key_event),
                        other => Some(other),
                    });
                }

                self.current_screen.handle_keyboard(key_event)
            }

            crossterm::event::Event::Mouse(mouse_event) => {
                if self.error_overlay.is_open() {
                    return self.error_overlay.handle_mouse(mouse_event);
                }

                if self.overlay.is_open() {
                    return self.overlay.handle_mouse(mouse_event);
                }

                if self.navbar.is_selected() {
                    return self.navbar.handle_mouse(mouse_event)
                        .and_then(|action| match action {
                            Action::NavbarSelect(_) => self.current_screen.handle_mouse(mouse_event),
                            other => Some(other),
                        });
                }

                self.current_screen.handle_mouse(mouse_event)
            }

            _ => { None }
        }

    }

    pub fn update_screen(&mut self, update: BackgroundUpdate) {
        if update.id == "popup" {
            self.overlay.apply_update(update);
            return;
        }

        if self.current_screen.get_name() == update.id {
            self.current_screen.apply_update(update);
        } else if let Some(screen) = self.screen_storage.get_mut(&update.id) {
            screen.apply_update(update);
        }
    }

    // change screen stores the previous screen if not specified otherwise
    // the current screen is removed from the storage if it exists, or created anew
    // this allows for screens to be swapped and their state to be preserved
    pub fn change_screen(&mut self, screen_name: &str) {
        if self.current_screen.should_store() {
            self.screen_storage.insert(
                self.current_screen.get_name(),
                self.current_screen.clone_box(),
            );
        }

        if let Some(screen) = self.screen_storage.remove(screen_name) {
            self.current_screen = screen;
        } else {
            self.current_screen = create_screen(screen_name, &self.passable_info);
        }

        self.cleanup_backgrounds();
        self.spawn_background();
    }

    pub fn spawn_background(&mut self) {
        if let Some(handle) = self.current_screen.background() {
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
    pub id: String,
    pub updates: HashMap<String, Box<dyn Any + Send + Sync>>,
}

#[allow(dead_code)]
impl BackgroundUpdate {
    pub fn new<S: Into<String>>(screen_id: S) -> Self {
        Self {
            id: screen_id.into(),
            updates: HashMap::new(),
        }
    }

    pub fn set<T: Any + Send + Sync, S: Into<String>>(mut self, field: S, value: T) -> Self {
        self.updates.insert(field.into(), Box::new(value));
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
        self.updates
            .remove(field)?
            .downcast::<T>()
            .ok()
            .map(|boxed| *boxed)
    }
}
