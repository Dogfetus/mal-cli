use ratatui::DefaultTerminal;
use crate::ui;
use std::io;

// store the screens the user is at.
pub enum CurrentScreen {
    Main,
    Anime,
    Manga,
    Info,
    Profile,
}

// here will all the details of a specific anime or manga be stored.
pub enum CurrentInfo {
    Anime,
    Manga,
}

pub struct App {
    pub key_input: String,              // the currently being edited json key.
    pub value_input: String,            // the currently being edited json value.

    pub current_screen: CurrentScreen,
    pub current_info: Option<CurrentInfo>,

    pub exit: bool,
}

impl App {
    pub fn new() -> App {
        App {
            key_input: String::new(),
            value_input: String::new(),

            current_screen: CurrentScreen::Main,
            current_info: None,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw( |frame| ui::draw(frame, self))?;
        }

        Ok(())
    }

}

