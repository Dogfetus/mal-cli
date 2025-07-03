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
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::symbols;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;

use super::widgets::animebox::LongAnimeBox;
use super::widgets::navbar::NavBar;
use super::widgets::navigatable::Navigatable;
use super::widgets::popup::AnimePopup;
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
    scroll_index: usize,
    selected_index: usize,
    image_manager: Arc<Mutex<ImageManager>>,

    navbar: NavBar,
    popup: AnimePopup,
    navigatable: Navigatable,
    focus: Focus,

    filter_popup: SelectionPopup,
    search_input: Input,

    fetching: bool,
    bg_sender: Option<Sender<LocalEvent>>,
    bg_loaded: bool,
}

impl SearchScreen {
    pub fn new() -> Self {
        Self {
            image_manager: Arc::new(Mutex::new(ImageManager::new())),
            navigatable: Navigatable::new((3, 4)),
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
            popup: AnimePopup::new(),
            navbar: NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(SEARCH)
                .add_screen(LIST)
                .add_screen(PROFILE),
            focus: Focus::Search,
            animes: Vec::new(),
            selected_index: 0,
            fetching: false,
            bg_loaded: false,
            scroll_index: 0,
            bg_sender: None,
        }
    }

    fn reset(&mut self) {
        self.animes.clear();
        self.scroll_index = 0;
        self.selected_index = 0;
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

    fn get_selected_anime(&self) -> Option<Anime> {
        if self.selected_index < self.animes.len() {
            Some(self.animes[self.selected_index].clone())
        } else {
            None
        }
    }
}

impl Screen for SearchScreen {
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
                .style(Style::default().fg(Color::DarkGray));
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

        let anime_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(anime_area);

        let search_field = Paragraph::new(self.search_input.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search")
                    .border_set(border::ROUNDED),
            )
            .style(style::Style::default().fg(if self.focus == Focus::Search {
                Color::Yellow
            } else {
                Color::DarkGray
            }));
        frame.render_widget(search_field, search_area);

        self.navigatable
            .construct(&self.animes, anime_area, |anime, area, highlight| {
                LongAnimeBox::render(anime, &self.image_manager, frame, area, highlight);
            });
        self.search_input
            .render_cursor(frame, search_area.x + 1, search_area.y + 1);
        self.filter_popup
            .render(frame, filter_area, self.focus == Focus::Filter);
        self.navbar.render(frame, top);
        self.popup.render(&self.image_manager, frame);
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
                        self.fetching = true;
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
                    self.popup.close();
                } else {
                    if self.popup.is_open() {
                        return self.popup.handle_input(key_event);
                    }

                    match key_event.code {
                        KeyCode::Char('k') | KeyCode::Down => {
                            self.navigatable.move_down(self.animes.len());
                        }
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.navigatable.move_up();
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.navigatable.move_right(self.animes.len());
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.navigatable.move_left();
                        }
                        KeyCode::Enter => {
                            if let Some(anime) = self.navigatable.get_selected_item(&self.animes) {
                                self.popup.set_anime(anime.clone());
                                self.popup.open();
                            }
                        }
                        _ => {}
                    }
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
        let image_manager = self.image_manager.clone();
        ImageManager::init_with_dedicated_thread(&image_manager, info.app_sx.clone(), id.clone());

        let handle = std::thread::spawn(move || {
            while let Ok(event) = bg_receiver.recv() {
                match event {
                    LocalEvent::FilterSwitch(filter_type) => {
                        if let Some(animes) = info.mal_client.get_top_anime(filter_type, 0, 100) {
                            let update =
                                super::BackgroundUpdate::new(id.clone()).set("animes", animes);
                            info.app_sx
                                .send(super::Event::BackgroundNotice(update))
                                .ok();
                        }
                    }

                    LocalEvent::Search(query) => {
                        if let Some(animes) = info.mal_client.search_anime(query.clone(), 0, 9) {
                            for anime in animes.iter() {
                                ImageManager::fetch_image_sequential(&image_manager, anime);
                            }
                            let update =
                                super::BackgroundUpdate::new(id.clone()).set("animes", animes);
                            info.app_sx
                                .send(super::Event::BackgroundNotice(update))
                                .ok();
                        }
                        if let Some(animes) = info.mal_client.search_anime(query, 10, 100) {
                            let update =
                                super::BackgroundUpdate::new(id.clone()).set("animes", animes);
                            info.app_sx
                                .send(super::Event::BackgroundNotice(update))
                                .ok();
                        }
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
