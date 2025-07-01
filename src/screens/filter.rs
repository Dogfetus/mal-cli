use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread::JoinHandle;

use crate::mal::models::anime::Anime;
use crate::utils::imageManager::ImageManager;
use crate::utils::input::Input;
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
use ratatui::widgets::Paragraph;

use super::widgets::animebox::AnimeBox;
use super::widgets::navbar::NavBar;
use super::widgets::popup::{Arrows, SelectionPopup};

#[derive(Debug, Clone)]
enum LocalEvent {
    FilterSwitch(String),
    Search(String),
}

#[derive(PartialEq, Debug, Clone)]
enum Focus {
    NavBar,
    Filter,
    Search,
    AnimeList,
}

#[derive(Clone)]
pub struct FilterScreen {
    navbar: super::widgets::navbar::NavBar,
    image_manager: Arc<Mutex<ImageManager>>,
    focus: Focus,

    filter_popup: SelectionPopup,
    search_input: Input,

    searching: bool,
    bg_sender: Option<Sender<LocalEvent>>,
    bg_loaded: bool,

}

impl FilterScreen {
    pub fn new() -> Self {
        Self {
            navbar: NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(FILTER)
                .add_screen(LIST)
                .add_screen(PROFILE),
            filter_popup: SelectionPopup::new()
                .with_arrows(Arrows::Static)
                .add_option("all")
                .add_option("airing")
                .add_option("upcoming")
                .add_option("tv")
                .add_option("ova")
                .add_option("movie")
                .add_option("special")
                .add_option("popularity")
                .add_option("favorite"),
            image_manager: Arc::new(Mutex::new(ImageManager::new())),
            focus: Focus::Search,
            search_input: Input::new(),
            searching: false,
            bg_sender: None,
            bg_loaded: false,
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

        let search_field = Paragraph::new(self.search_input.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search")
                    .border_set(symbols::border::ROUNDED),
            )
            .style(style::Style::default().fg(if self.focus == Focus::Search {
                style::Color::Yellow
            } else {
                style::Color::LightBlue
            }));

        frame.render_widget(search_field, search_area);

        AnimeBox::render(&Anime::example(), &self.image_manager, frame, one, false);
        self.search_input
            .render_cursor(frame, search_area.x + 1, search_area.y + 1);
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
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.focus = Focus::NavBar;
                            self.filter_popup.close();
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.focus = Focus::Search;
                            self.filter_popup.close();
                        }
                        _ => {}
                    }
                } else {
                    if let Some(filter_type) = self.filter_popup.handle_input(key_event) {
                        if let Some(sender) = &self.bg_sender {
                            sender.send(LocalEvent::FilterSwitch(filter_type)).ok();
                        }
                    }
                }
            }

            Focus::Search => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.navbar.select();
                            self.focus = Focus::NavBar;
                            return None;
                        }
                        KeyCode::Char('k') | KeyCode::Down => {
                            self.focus = Focus::AnimeList;
                            return None;
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.focus = Focus::Filter;
                            return None;
                        }
                        _ => {}
                    }
                }

                if let Some(text) = self.search_input.handle_event(key_event) {
                    if !text.is_empty() {
                        self.searching = true;
                        if let Some(sender) = &self.bg_sender {
                            sender.send(LocalEvent::Search(text)).ok();
                        }
                    }
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

            Focus::NavBar => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('k') | KeyCode::Down => {
                            self.navbar.deselect();
                            self.focus = Focus::Search
                        }
                        _ => {}
                    }
                } else {
                    return self.navbar.handle_input(key_event);
                }
            }
        }

        None
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
    }

    fn background(&mut self, info: super::BackgroundInfo) -> Option<JoinHandle<()>> {
        if self.bg_loaded {
            return None;
        }

        self.bg_loaded = true;
        let (bg_sender, bg_receiver) = channel::<LocalEvent>();
        self.bg_sender = Some(bg_sender);
        let id = self.get_name();

        let handle = std::thread::spawn(move || {
            while let Ok(event) = bg_receiver.recv() {
                match event {
                    LocalEvent::FilterSwitch(filter_type) => {
                        if let Some(animes) = info.mal_client.get_top_anime(filter_type, 0, 100) {
                            let update = super::BackgroundUpdate::new(id.clone())
                                .set("animes", animes);
                            info.app_sx
                                .send(super::Event::BackgroundNotice(update))
                                .ok();
                        }
                    }
                    LocalEvent::Search(query) => {
                    }
                }
            }
        });
        Some(handle)
    }

    fn apply_update(&mut self, update: super::BackgroundUpdate) {
        if let Some(fetching) = update.get::<String>("test") {
        }
    }
}
