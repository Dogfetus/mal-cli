use ratatui::Frame;

use crate::app::App;
use crate::app::CurrentScreen;

mod main;
mod info;
mod list;
mod profile;
mod settings;


// when adding new screens add them here and also in the enum in app.rs
pub fn draw(frame: &mut Frame, app: &App) { 
    match app.current_screen {
        CurrentScreen::Main => main::draw(frame, app),
        CurrentScreen::Anime => info::draw(frame, app),
        CurrentScreen::Manga => info::draw(frame, app),
        CurrentScreen::Info => info::draw(frame, app),
        CurrentScreen::Profile => profile::draw(frame, app),
        CurrentScreen::Settings => settings::draw(frame, app),
    }
}
