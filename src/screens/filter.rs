use std::sync::Arc;
use std::sync::Mutex;

use crate::mal::models::anime::Anime;
use crate::utils::imageManager::ImageManager;
use crate::{app::Action, screens::Screen, screens::screens::*};
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use crossterm::event::KeyEvent;
use crossterm::event::KeyCode;
use ratatui::widgets::Borders;
use ratatui::widgets::Block;
use ratatui::layout::Layout;
use ratatui::style;
use ratatui::widgets::Clear;
use ratatui::Frame;

use super::widgets::animebox;
use super::widgets::animebox::AnimeBox;
use super::widgets::navbar::NavBar;


#[derive(Clone)]
pub struct FilterScreen {
    navbar: super::widgets::navbar::NavBar,
    selected_button: usize,
    buttons: Vec<&'static str>,
    image_manager: Arc<Mutex<ImageManager>>,
}

impl FilterScreen {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            buttons: vec![
                "Back",
                "Exit",
            ],
            navbar: NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(FILTER)
                .add_screen(LIST)
                .add_screen(PROFILE),
            image_manager: Arc::new(Mutex::new(ImageManager::new())),
        }
    }
}

impl Screen for FilterScreen {
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        /* Splitting the screen:
         * which looks like this:
         * ╭────────╮
         * ╰────────╯
         * ╭─────╮╭─╮
         * ╰─────╯│ │
         * ╭─────╮│ │
         * │     ││ │
         * ╰─────╯╰─╯
         * */
        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .areas(area);

        let [bottom_top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .areas(bottom);

        let [_, bottom_middle, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .areas(bottom);

        let [one, two, three, four] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .areas(bottom_middle);

        let block = Block::new()
            .borders(Borders::ALL)
            .style(style::Style::default().fg(style::Color::LightBlue));

        frame.render_widget(block.clone(), one);
        frame.render_widget(block.clone(), two);
        frame.render_widget(block.clone(), three);
        frame.render_widget(block.clone(), four);

        AnimeBox::render(&Anime::example(), &self.image_manager, frame, one, false);

        self.navbar.render(frame, top);
    }


    // returns an action based on the input that the app will act upon
    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        None
        // code to handle inputs from user
        // ...
        // ...
        // ...
        // example:
        // match key_event.code {
        //     KeyCode::Up | KeyCode::Char('j') => {}
        //     KeyCode::Down | KeyCode::Char('k') => {}
        //     KeyCode::Left | KeyCode::Char('h') => {}
        //     KeyCode::Right | KeyCode::Char('l') => {}
        //     KeyCode::Enter => {}
        //     _ => {} 
        // };
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
    }

    fn background(&mut self, info: super::BackgroundInfo) -> Option<std::thread::JoinHandle<()>> {
        None
        // code to start a background thread
        // ...
        // ...
        // ...
        // example:
        // let handle = std::thread::spawn(move || {
        //     // background functionality here
        //     // use info.app_sx to send events to the app
        // });
        // Some(handle)
    }
}




