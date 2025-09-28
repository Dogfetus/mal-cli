use super::ExtraInfo;
use super::widgets::animebox::AnimeBox;
use super::widgets::navigatable::Navigatable;
use super::widgets::popup::{Arrows, SelectionPopup};
use crate::add_screen_caching;
use crate::app::Event;
use crate::config::HIGHLIGHT_COLOR;
use crate::config::PRIMARY_COLOR;
use crate::mal::models::anime::Anime;
use crate::mal::models::anime::AnimeId;
use crate::utils::functionStreaming::StreamableRunner;
use crate::utils::imageManager::ImageManager;
use crate::utils::input::Input;
use crate::{app::Action, screens::Screen};
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use image::imageops::filter3x3;
use ratatui::Frame;
use ratatui::layout::{Alignment, Position};
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
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread::JoinHandle;

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
    animes: Vec<AnimeId>,
    image_manager: Arc<Mutex<ImageManager>>,
    app_info: ExtraInfo,

    navigatable: Navigatable,
    focus: Focus,

    filter_popup: SelectionPopup,
    search_input: Input,
    search_area: Option<ratatui::layout::Rect>,

    fetching: bool,
    bg_sender: Option<Sender<LocalEvent>>,
    bg_loaded: bool,
}

impl SearchScreen {
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
            focus: Focus::NavBar,
            animes: Vec::new(),
            search_area: None,
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

    fn fetch_and_send_animes<F>(app_sx: &Sender<Event>, id: String, fetch_fn: F)
    where
        F: FnMut(usize, usize) -> Option<Vec<Anime>>,
    {
        let anime_generator = StreamableRunner::new()
            .change_batch_size_at(100, 1)
            .stop_at(2);

        for animes in anime_generator.run(fetch_fn) {
            let anime_ids = animes
                .iter()
                .map(|anime| anime.id)
                .collect::<Vec<AnimeId>>();
            let update = super::BackgroundUpdate::new(id.clone())
                .set("animes", animes)
                .set("anime_ids", anime_ids);
            app_sx.send(super::Event::BackgroundNotice(update)).ok();
        }
    }
}

impl Screen for SearchScreen {
    add_screen_caching!();

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let [_top, bottom] = Layout::default()
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
            .construct(&self.animes, anime_area, |anime_id, area, highlight| {
                if let Some(anime) = self.app_info.anime_store.get(anime_id) {
                    AnimeBox::render(
                        &anime,
                        &self.image_manager,
                        frame,
                        area,
                        highlight && self.focus == Focus::AnimeList,
                    );
                }
            });
        self.search_area = Some(search_area);
        self.search_input.render_cursor(
            frame,
            search_area.x + 1,
            search_area.y + 1,
            self.focus == Focus::Search,
        );
        self.filter_popup
            .render(frame, filter_area, self.focus == Focus::Filter);
    }

    fn handle_keyboard(&mut self, key_event: KeyEvent) -> Option<Action> {
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
                            return Some(Action::NavbarSelect(true));
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.focus = Focus::Search;
                            self.filter_popup.close();
                        }
                        _ => {}
                    }
                } else if let Some(mut filter_type) = self.filter_popup.handle_input(key_event) {
                    self.fetching = true;
                    if filter_type == "popularity" {
                        filter_type = "bypopularity".to_string();
                    }
                    if let Some(sender) = &self.bg_sender {
                        sender.send(LocalEvent::FilterSwitch(filter_type)).ok();
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
                            return Some(Action::NavbarSelect(true));
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
                            if let Some(anime_id) = self.navigatable.get_selected_item(&self.animes)
                            {
                                if let Some(anime) = self.app_info.anime_store.get(anime_id) {
                                    return Some(Action::ShowOverlay(anime.id));
                                }
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

    fn handle_mouse(&mut self, mouse_event: crossterm::event::MouseEvent) -> Option<Action> {
        if mouse_event.row < 3 {
            self.focus = Focus::NavBar;
            return Some(Action::NavbarSelect(true));
        }

        if let Some(search_area) = self.search_area {
            let pos = Position::new(mouse_event.column, mouse_event.row);
            if search_area.contains(pos) {
                self.focus = Focus::Search;
                return None;
            }
        }

        if let Some(filter_area) = self.filter_popup.get_area(){
            let pos = Position::new(mouse_event.column, mouse_event.row);
            if filter_area.contains(pos) {
                self.focus = Focus::Filter;
            }
            if let Some(filter) = self.filter_popup.handle_mouse(mouse_event){
            }
        }

        if let Some(index) = self.navigatable.get_hovered_index(mouse_event) {
            self.navigatable.set_selected_index(index);
            self.focus = Focus::AnimeList;

            if let crossterm::event::MouseEventKind::Down(_) = mouse_event.kind {
                let anime_id = self.navigatable.get_selected_item(&self.animes)?;
                return Some(Action::ShowOverlay(*anime_id));
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
        let mal_client = info.mal_client.clone();
        let app_sx = info.app_sx.clone();

        let handle = std::thread::spawn(move || {
            if nr_of_animes == 0 {
                Self::fetch_and_send_animes(&app_sx, id.clone(), |offset, limit| {
                    mal_client.get_top_anime("all".to_string(), offset, limit)
                });
            }

            while let Ok(event) = bg_receiver.recv() {
                match event {
                    LocalEvent::FilterSwitch(filter_type) => {
                        Self::fetch_and_send_animes(&app_sx, id.clone(), |offset, limit| {
                            mal_client.get_top_anime(filter_type.clone(), offset, limit)
                        });
                    }

                    LocalEvent::Search(query) => {
                        Self::fetch_and_send_animes(&app_sx, id.clone(), |offset, limit| {
                            info.mal_client.search_anime(query.clone(), offset, limit)
                        });
                    }
                }
            }
        });
        Some(handle)
    }

    fn apply_update(&mut self, mut update: super::BackgroundUpdate) {
        if let Some(ids) = update.take::<Vec<AnimeId>>("anime_ids") {
            if self.fetching {
                self.reset();
            }
            self.animes.extend(ids);
        }
    }
}
