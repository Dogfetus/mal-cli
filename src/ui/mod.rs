use ratatui::Frame;

use crate::app::App;
use crate::app::CurrentScreen;

mod main;
mod info;
mod list;
mod profile;

pub fn draw(frame: &mut Frame, app: &App) { 
    match app.current_screen {
        CurrentScreen::Main => main::draw(frame, app),
        CurrentScreen::Anime => info::draw(frame, app),
        CurrentScreen::Manga => info::draw(frame, app),
        CurrentScreen::Info => info::draw(frame, app),
        CurrentScreen::Profile => profile::draw(frame, app),
        _ => {},
    }
}





// pub const DRAW_FUNCTIONS: [DrawFn; 5] = [
//     home::draw,
//     info::draw,
//     list::draw,
//     profile::draw,
//
// ];
