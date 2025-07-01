use std::sync::Arc;
use std::sync::Mutex;

use crate::mal::models::anime::Anime;
use crate::utils::imageManager::ImageManager;
use crate::{app::Action, screens::Screen, screens::screens::*};
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style;
use ratatui::symbols;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;

use super::widgets::animebox::AnimeBox;
use super::widgets::navbar::NavBar;
use super::widgets::popup::SelectionPopup;

#[derive(PartialEq, Debug, Clone)]
enum Focus {
    Filter,
    Search,
    AnimeList,
}

#[derive(Clone)]
pub struct FilterScreen {
    navbar: super::widgets::navbar::NavBar,
    selected_button: usize,
    buttons: Vec<&'static str>,
    image_manager: Arc<Mutex<ImageManager>>,
    filter_type: String,
    filter_popup: SelectionPopup,
    focus: Focus,
}

impl FilterScreen {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            buttons: vec!["Back", "Exit"],
            navbar: NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(FILTER)
                .add_screen(LIST)
                .add_screen(PROFILE),
            filter_popup: SelectionPopup::new()
                .add_option("all")
                .add_option("airing")
                .add_option("upcoming")
                .add_option("tv")
                .add_option("ova")
                .add_option("movie")
                .add_option("special")
                .add_option("bypopularity")
                .add_option("favorite"),
            image_manager: Arc::new(Mutex::new(ImageManager::new())),
            filter_type: String::new(),
            focus: Focus::Search,
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

        let [_, bottom_middle, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .areas(bottom);

        let [search_area, _, anime_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(bottom_middle);

        let [search_area, filter_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .areas(search_area);

        let [one, two, three, four] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .areas(anime_area);

        let block = Block::new()
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .style(
                style::Style::default().fg(if self.focus == Focus::AnimeList {
                    style::Color::Yellow
                } else {
                    style::Color::LightBlue
                }),
            );

        let search_block = Block::new()
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .title("Search")
            .style(style::Style::default().fg(if self.focus == Focus::Search {
                style::Color::Yellow
            } else {
                style::Color::LightBlue
            }));

        frame.render_widget(search_block, search_area);
        frame.render_widget(block.clone(), one);
        frame.render_widget(block.clone(), two);
        frame.render_widget(block.clone(), three);
        frame.render_widget(block.clone(), four);

        AnimeBox::render(&Anime::example(), &self.image_manager, frame, one, false);

        self.filter_popup
            .render(frame, filter_area, self.focus == Focus::Filter);
        self.navbar.render(frame, top);
    }

    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match self.focus {
            Focus::Filter => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('k') | KeyCode::Down => {
                            self.focus = Focus::AnimeList;
                            self.filter_popup.close();
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.focus = Focus::Search;
                            self.filter_popup.close();
                        }
                        _ => {}
                    }
                } else {
                    self.filter_popup.handle_input(key_event);
                }
            }
            Focus::Search => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('k') | KeyCode::Down => self.focus = Focus::AnimeList,
                        KeyCode::Char('l') | KeyCode::Right => self.focus = Focus::Filter,
                        _ => {}
                    }
                } else {
                }
            }
            Focus::AnimeList => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Up => self.focus = Focus::Search,
                        _ => {}
                    }
                } else {
                }
            }
        }

        None
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
