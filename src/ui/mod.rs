use ratatui::Frame;

use crate::app::App;
use crate::app::CurrentScreen;

mod launch;
mod info;
mod list;
mod profile;
mod settings;
mod overview;



// TODO: make these screens structs and implement the trait for them.
// TODO: they should take care of their own buttons and such
// TODO: instead of mapping each screen, figure out how to do currentscreen.render with traits
// TODO: https://claude.ai/chat/90b1b44c-d63a-4589-8228-86573cc1c23f <- this 
// when adding new screens add them here and also in the enum in app.rs
pub fn draw(frame: &mut Frame, app: &App) { 

    #[allow(unreachable_patterns)]
    match app.current_screen {
        CurrentScreen::Launch => launch::draw(frame, app),
        CurrentScreen::Anime => info::draw(frame, app),
        CurrentScreen::Manga => info::draw(frame, app),
        CurrentScreen::Info => info::draw(frame, app),
        CurrentScreen::Profile => profile::draw(frame, app),
        CurrentScreen::Settings => settings::draw(frame, app),
        CurrentScreen::Overview => overview::draw(frame, app),
        _ => launch::draw(frame, app), 
    }
}



// pub trait Screen {
//     fn draw(&self, frame: &mut Frame, app: &App);
//     fn handle_input(&mut self, app: &mut App);
//     fn update(&mut self, app: &mut App);
//     fn render(&self, frame: &mut Frame, app: &App);
// }
