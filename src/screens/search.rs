use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread::JoinHandle;

//TODO: remember to fetch all search results and also fetch list of animes when going to this
//screen

use crate::add_screen_caching;
use crate::app::Event;
use crate::config::HIGHLIGHT_COLOR;
use crate::config::PRIMARY_COLOR;
use crate::mal::models::anime::Anime;
use crate::utils::functionStreaming::StreamableRunner;
use crate::utils::imageManager::ImageManager;
use crate::utils::input::Input;
use crate::{app::Action, screens::Screen, screens::screens::*};
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style;
use ratatui::style::Style;
use ratatui::symbols;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;

use super::ExtraInfo;
use super::widgets::animebox::LongAnimeBox;
use super::widgets::navbar::NavBar;
use super::widgets::navigatable::Navigatable;
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
pub struct SearchScreen {
    animes: Vec<Anime>,
    image_manager: Arc<Mutex<ImageManager>>,
    app_info: ExtraInfo,

    navigatable: Navigatable,
    focus: Focus,

    filter_popup: SelectionPopup,
    search_input: Input,

    fetching: bool,
    bg_sender: Option<Sender<LocalEvent>>,
    bg_loaded: bool,
}

impl SearchScreen {
    add_screen_caching!();

    pub fn new(info: ExtraInfo) -> Self {
        Self {
            image_manager: Arc::new(Mutex::new(ImageManager::new())),
            navigatable: Navigatable::new((3, 2)),
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
            search_input: Input::new(),
            focus: Focus::Search,
            animes: Vec::new(),
            bg_loaded: false,
            bg_sender: None,
            fetching: false,
            app_info: info,
        }
    }

    fn reset(&mut self) {
        self.navigatable.back_to_start();
        self.animes.clear();
        self.fetching = false;
    }

    fn int_length(&self, mut n: usize) -> usize {
        if n == 0 {
            return 1;
        }
        let mut len = 0;
        while n > 0 {
            n /= 10;
            len += 1;
        }
        len
    }

    fn fetch_and_send_animes<F>(context: &ExtraInfo, id: String, fetch_fn: F)
    where
        F: FnMut(usize, usize) -> Option<Vec<Anime>>,
    {
        let anime_generator = StreamableRunner::new()
            .change_batch_size_at(100, 1)
            .stop_at(2);

        for animes in anime_generator.run(fetch_fn) {
            let update = super::BackgroundUpdate::new(id.clone()).set("animes", animes);
            context
                .app_sx
                .send(super::Event::BackgroundNotice(update))
                .ok();
        }
    }
}

impl Screen for SearchScreen {
    add_screen_caching!();

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .areas(area);

        let [result_area, bottom_middle, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .areas(bottom);

        if !self.animes.is_empty() {
            let width = self.int_length(self.animes.len()) as u16 + 4;

            let [_, result_area, _] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Length(width + 4),
                    Constraint::Fill(1),
                ])
                .areas(result_area);

            let [result_area, _] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Fill(1)])
                .areas(result_area);

            let results = Paragraph::new(self.animes.len().to_string())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_set(symbols::border::ROUNDED),
                )
                .style(Style::default().fg(PRIMARY_COLOR));
            frame.render_widget(results, result_area);
        }

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

        let search_field = Paragraph::new(self.search_input.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search")
                    .border_set(border::ROUNDED),
            )
            .style(style::Style::default().fg(if self.focus == Focus::Search {
                HIGHLIGHT_COLOR
            } else {
                PRIMARY_COLOR
            }));
        frame.render_widget(search_field, search_area);

        self.navigatable
            .construct(&self.animes, anime_area, |anime, area, highlight| {
                LongAnimeBox::render(
                    anime,
                    &self.image_manager,
                    frame,
                    area,
                    highlight && self.focus == Focus::AnimeList,
                );
            });
        self.search_input
            .render_cursor(frame, search_area.x + 1, search_area.y + 1);
        self.filter_popup
            .render(frame, filter_area, self.focus == Focus::Filter);
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
                            return Some(Action::NavbarSelect(true))
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.focus = Focus::Search;
                            self.filter_popup.close();
                        }
                        _ => {}
                    }
                } else {
                    if let Some(mut filter_type) = self.filter_popup.handle_input(key_event) {
                        self.fetching = true;
                        if filter_type == "popularity" {
                            filter_type = "bypopularity".to_string();
                        }
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
                            self.focus = Focus::NavBar;
                            return Some(Action::NavbarSelect(true))
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

                if let Some(text) = self.search_input.handle_event(key_event, false) {
                    if !text.is_empty() {
                        self.fetching = true;
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
                    match key_event.code {
                        KeyCode::Char('k') | KeyCode::Down => {
                            self.navigatable.move_down();
                        }
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.navigatable.move_up();
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.navigatable.move_right();
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.navigatable.move_left();
                        }
                        KeyCode::Enter => {
                            if let Some(anime) = self.navigatable.get_selected_item(&self.animes) {
                                return Some(Action::ShowOverlay(anime.clone()));
                            }
                        }
                        _ => {}
                    }
                }
            }

            Focus::NavBar => {
                self.focus = Focus::Search;
            }
        }

        None
    }

    fn background(&mut self) -> Option<JoinHandle<()>> {
        if self.bg_loaded {
            return None;
        }

        let info = self.app_info.clone();
        let nr_of_animes = self.animes.len();
        self.bg_loaded = true;
        let (bg_sender, bg_receiver) = channel::<LocalEvent>();
        self.bg_sender = Some(bg_sender);
        let id = self.get_name();
        let image_manager = self.image_manager.clone();
        ImageManager::init_with_threads(&image_manager, info.app_sx.clone());

        let handle = std::thread::spawn(move || {
            if nr_of_animes <= 0 {
                Self::fetch_and_send_animes(&info, id.clone(), |offset, limit| {
                    info.mal_client
                        .get_top_anime("all".to_string(), offset, limit)
                });
            }

            while let Ok(event) = bg_receiver.recv() {
                match event {
                    LocalEvent::FilterSwitch(filter_type) => {
                        Self::fetch_and_send_animes(&info, id.clone(), |offset, limit| {
                            info.mal_client
                                .get_top_anime(filter_type.clone(), offset, limit)
                        });
                    }

                    LocalEvent::Search(query) => {
                        Self::fetch_and_send_animes(&info, id.clone(), |offset, limit| {
                            info.mal_client.search_anime(query.clone(), offset, limit)
                        });
                    }
                }
            }
        });
        Some(handle)
    }

    fn apply_update(&mut self, mut update: super::BackgroundUpdate) {
        if let Some(animes) = update.take::<Vec<Anime>>("animes") {
            if self.fetching {
                self.reset();
            }
            self.animes.extend(animes);
        }
    }
}
