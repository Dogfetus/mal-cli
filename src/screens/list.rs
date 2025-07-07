use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::app::Event;
use crate::mal::models::anime::Anime;
use crate::utils::imageManager::ImageManager;
use crate::utils::input::Input;
use crate::{app::Action, screens::Screen};
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::{Alignment, Constraint, Margin, Rect};
use ratatui::style;
use ratatui::style::Color;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;

use super::widgets::animebox::LongAnimeBox;
use super::widgets::navbar::NavBar;
use super::widgets::navigatable::Navigatable;
use super::widgets::popup::{AnimePopup, Arrows};
use super::widgets::popup::SelectionPopup;
use super::{BackgroundUpdate, screens::*};

#[derive(Debug, Clone)]
struct Statistics {
    pub total_animes: usize,
    pub animes_in_list: usize,
    pub filteres_animes: usize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            total_animes: 0,
            animes_in_list: 0,
            filteres_animes: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Filters {
    list_type: String,
    airing_type: String,
    anime_type: String,
    sort_by: String,
    sort_order: String,
}

impl Filters {
    fn new() -> Self {
        Self {
            list_type: "all".to_string(),
            airing_type: "all".to_string(),
            anime_type: "all".to_string(),
            sort_by: "by title".to_string(),
            sort_order: "ascending".to_string(),
        }
    }

    fn update(&mut self, index: usize, value: String) {
        match index {
            0 => self.list_type = value,
            1 => self.airing_type = value,
            2 => self.anime_type = value,
            3 => self.sort_by = value,
            4 => self.sort_order = value,
            _ => {}
        }
    }
}

enum LocalEvent {
    Dropdown(Vec<Anime>, Filters),
    Search(Vec<Anime>, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    NavBar,
    Content,
    Search,
    Dropdown,
}

#[derive(Clone)]
pub struct ListScreen {
    all_animes: Vec<Anime>,
    filtered_animes: Vec<Anime>,
    filters: Filters,
    statistics: Statistics,

    bg_loaded: bool,
    bg_sx: Option<Sender<LocalEvent>>,
    bg_startup: bool,
    image_manager: Arc<Mutex<ImageManager>>,

    focus: Focus,
    navbar: NavBar,
    popup: AnimePopup,

    search_input: Input,
    navigatable: Navigatable,
    dropdowns: Vec<SelectionPopup>,
    dropdown_nav: Navigatable,
}

impl ListScreen {
    pub fn new() -> Self {
        Self {
            image_manager: Arc::new(Mutex::new(ImageManager::new())),
            navigatable: Navigatable::new((3, 3)),
            dropdown_nav: Navigatable::new((5, 1)),
            dropdowns: vec![
                SelectionPopup::new()
                    .with_arrows(Arrows::Static)
                    .add_option("all")
                    .add_option("watching")
                    .add_option("plan to watch")
                    .add_option("completed")
                    .add_option("on hold")
                    .add_option("dropped"),
                SelectionPopup::new()
                    .with_arrows(Arrows::Static)
                    .add_option("all")
                    .add_option("airing")
                    .add_option("upcoming")
                    .add_option("finished"),
                SelectionPopup::new()
                    .with_arrows(Arrows::Static)
                    .add_option("all")
                    .add_option("tv")
                    .add_option("movie")
                    .add_option("ova")
                    .add_option("ona")
                    .add_option("special"),
                SelectionPopup::new()
                    .with_arrows(Arrows::Static)
                    .add_option("sort")
                    .add_option("by title")
                    .add_option("by score")
                    .add_option("by last updated")
                    .add_option("by episodes")
                    .add_option("by popularity")
                    .add_option("by start date")
                    .add_option("by end date"),
                SelectionPopup::new()
                    .with_arrows(Arrows::Static)
                    .add_option("ascending")
                    .add_option("descending"),
            ],
            statistics: Statistics::new(),
            search_input: Input::new(),
            popup: AnimePopup::new(),
            navbar: NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(SEARCH)
                .add_screen(LIST)
                .add_screen(PROFILE),
            filters: Filters::new(),
            focus: Focus::Content,
            all_animes: Vec::new(),
            filtered_animes: Vec::new(),
            bg_startup: true,
            bg_loaded: false,
            bg_sx: None,
        }
    }

    fn sort_animes(animes: &mut Vec<Anime>, sort_by: &str, order: &str) {
        match sort_by {
            "by title" => {
                animes.sort_by(|a, b| a.title.cmp(&b.title));
            }
            "by score" => {
                animes.sort_by(|a, b| {
                    a.mean
                        .partial_cmp(&b.mean)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            "by episodes" => {
                animes.sort_by(|a, b| a.num_episodes.cmp(&b.num_episodes));
            }
            "by popularity" => {
                animes.sort_by(|a, b| a.popularity.cmp(&b.popularity));
            }
            "by start date" => {
                animes.sort_by(|a, b| a.start_date.cmp(&b.start_date));
            }
            "by end date" => {
                animes.sort_by(|a, b| a.end_date.cmp(&b.end_date));
            }
            _ => {}
        }

        if order == "descending" {
            animes.reverse();
        }
    }

    fn filter_animes(animes: &mut Vec<Anime>, filters: &Filters) {
        if filters.list_type != "all" {
            animes.retain(|anime| anime.my_list_status.status == filters.list_type);
        }

        if filters.airing_type != "all" {
            animes.retain(|anime| anime.status == filters.airing_type);
        }

        // Filter by anime type
        if filters.anime_type != "all" {
            animes.retain(|anime| anime.media_type == filters.anime_type);
        }

        Self::sort_animes(animes, &filters.sort_by, &filters.sort_order);
    }

    fn search_animes(animes: &mut Vec<Anime>, search: String) {
        if search.is_empty() {
            return;
        }

        let search_lower = search.to_lowercase();
        animes.retain(|anime| {
            anime.title.to_lowercase().contains(&search_lower)||
            anime.alternative_titles.en.to_lowercase().contains(&search_lower) ||
            anime.alternative_titles.ja.to_lowercase().contains(&search_lower) ||
            anime.alternative_titles.synonyms.iter().any(|syn| syn.to_lowercase().contains(&search_lower))
        });
    }
}

impl Screen for ListScreen {
    // draws the screen
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .areas(area);

        let [side, bottom, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Fill(1),
                Constraint::Percentage(20),
            ])
            .areas(bottom);

        let [search, _, content] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(bottom);

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
        frame.render_widget(search_field, search);

        let info_area = Rect::new(
            side.x + ((side.width + 1) / 2) - ((side.width + 1) * 4 / 10),
            content.y,
            (side.width + 1) * 8 / 10,
            content.height * 3 / 10,
        );

        let dropdown_area = Rect::new(
            top.x + top.width - (side.width + 1) / 2 - info_area.width / 2,
            info_area.y,
            info_area.width,
            info_area.height.max(self.dropdowns.len() as u16 * 3)
        );

        let [info_area_left, info_area_right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .areas(info_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .style(style::Style::default().fg(Color::DarkGray));
        frame.render_widget(block, info_area);

        let info = Paragraph::new(" Animes found:\n Selected list:\n")
            .block(Block::default().borders(Borders::TOP).title("Info"))
            .alignment(Alignment::Left)
            .style(style::Style::default().fg(Color::DarkGray));

        let info_value = Paragraph::new(format!("{}\n0\n", self.filtered_animes.len()))
            .alignment(Alignment::Left)
            .style(style::Style::default().fg(Color::DarkGray));
        frame.render_widget(info, info_area_left.inner(Margin::new(1, 0)));
        frame.render_widget(info_value, info_area_right.inner(Margin::new(1, 1)));

        self.navigatable
            .construct(&self.filtered_animes, content, |anime, area, highlight| {
                LongAnimeBox::render(
                    anime,
                    &self.image_manager,
                    frame,
                    area,
                    highlight && self.focus == Focus::Content,
                );
            });

        self.dropdown_nav.construct(
            &self.dropdowns,
            dropdown_area,
            |dropdown, area, highlight| {
                dropdown.render(frame, area, highlight && self.focus == Focus::Dropdown);
            },
        );

        self.navbar.render(frame, top);
        self.popup.render(&self.image_manager, frame);
    }

    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match self.focus {
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
                            self.focus = Focus::Content;
                            return None;
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.focus = Focus::Dropdown;
                            return None;
                        }
                        _ => {}
                    }
                }

                if let Some(text) = self.search_input.handle_event(key_event, true) {
                    if let Some(sx) = &self.bg_sx {
                        sx.send(LocalEvent::Search(self.all_animes.clone(), text)).ok();
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
                            self.focus = Focus::Search;
                        }
                        _ => {}
                    }
                } else {
                    return self.navbar.handle_input(key_event);
                }
            }

            Focus::Content => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.focus = Focus::Search;
                            return None;
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.focus = Focus::Dropdown;
                            return None;
                        }
                        _ => {}
                    }
                    return None;
                } 

                if self.popup.is_open() {
                    return self.popup.handle_input(key_event);
                }

                match key_event.code {
                    KeyCode::Char('j') | KeyCode::Up => {
                        self.navigatable.move_up();
                    }
                    KeyCode::Char('k') | KeyCode::Down => {
                        self.navigatable.move_down();
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        self.navigatable.move_right();
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        self.navigatable.move_left();
                    }
                    KeyCode::Enter => {
                        if let Some(anime) = self.navigatable.get_selected_item(&self.filtered_animes)
                        {
                            self.popup.set_anime(anime.clone());
                            self.popup.open();
                        }
                    }
                    _ => {}
                }
            }

            Focus::Dropdown => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('h') | KeyCode::Left | KeyCode::Char('k') | KeyCode::Down => {
                            self.focus = Focus::Content;
                            if let Some(dropdown) =
                                self.dropdown_nav.get_selected_item_mut(&mut self.dropdowns)
                            {
                                dropdown.close();
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.focus = Focus::Search;
                            if let Some(dropdown) =
                                self.dropdown_nav.get_selected_item_mut(&mut self.dropdowns)
                            {
                                dropdown.close();
                            }
                            return None;
                        }
                        _ => {}
                    }
                    return None;
                }

                if let Some(dropdown) = self.dropdown_nav.get_selected_item_mut(&mut self.dropdowns)
                {
                    if !dropdown.is_open() {
                        if matches!(key_event.code, KeyCode::Char('j') | KeyCode::Up) {
                            self.dropdown_nav.move_up();
                            return None;
                        } else if matches!(key_event.code, KeyCode::Char('k') | KeyCode::Down) {
                            self.dropdown_nav.move_down();
                            return None;
                        }
                    }

                    if let Some(selection) = dropdown.handle_input(key_event) {
                        let index = self.dropdown_nav.get_selected_index();
                        self.filters.update(index, selection);

                        if let Some(sx) = &self.bg_sx {
                            sx.send(LocalEvent::Dropdown(self.all_animes.clone(), self.filters.clone())).ok();
                        }
                    }
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

        let id = self.get_name();
        let (sx, rx) = channel::<LocalEvent>();
        self.bg_sx = Some(sx);
        ImageManager::init_with_dedicated_thread(
            &self.image_manager,
            info.app_sx.clone(),
            id.clone(),
        );

        Some(std::thread::spawn(move || {
            let mut offset = 0;
            let mut limit = 10;
            let mut cached_filter = Option::<Filters>::None;
            let mut cached_search = String::new();

            // fetch enitre list of anime for the user
            loop {
                if let Some(animes) = info.mal_client.get_anime_list(None, offset, limit) {
                    let nr_of_animes = animes.len();

                    let update = BackgroundUpdate::new(id.clone())
                        .set("animes", animes)
                        .set("extend", true);
                    info.app_sx.send(Event::BackgroundNotice(update)).ok();

                    if nr_of_animes < limit as usize {
                        break;
                    }

                    offset += limit;
                    limit = 1000;
                } else {
                    break;
                }
            }

            let update = BackgroundUpdate::new(id.clone()).set("startup", false);
            info.app_sx.send(Event::BackgroundNotice(update)).ok();

            while let Ok(_event) = rx.recv() {
                match _event {
                    LocalEvent::Dropdown(animes, filters) => {
                        cached_filter = Some(filters.clone());
                        let mut filtered_animes = animes;
                        Self::filter_animes(&mut filtered_animes, &filters);

                        if !cached_search.is_empty(){
                            Self::search_animes(&mut filtered_animes, cached_search.clone());
                        }

                        let update =
                            BackgroundUpdate::new(id.clone()).set("filtered_animes", filtered_animes);
                        info.app_sx.send(Event::BackgroundNotice(update)).ok();
                    }

                    LocalEvent::Search(animes, search) => {
                        let mut latest_search = search;
                        let mut latest_animes = animes;

                        while let Ok(_event) = rx.recv_timeout(Duration::from_millis(250)) {
                            if let LocalEvent::Search(animes, search) = _event {
                                latest_search = search;
                                latest_animes = animes;
                            }
                        }

                        if let Some(filters) = cached_filter.clone() {
                            Self::filter_animes(&mut latest_animes, &filters);
                        }

                        cached_search = latest_search.clone();
                        Self::search_animes(&mut latest_animes, latest_search);
                        let update = BackgroundUpdate::new(id.clone())
                            .set("filtered_animes", latest_animes);
                        info.app_sx.send(Event::BackgroundNotice(update)).ok();
                    }
                }
            }
        }))
    }

    fn apply_update(&mut self, mut update: super::BackgroundUpdate) {
        match (
            update.take::<Vec<Anime>>("animes"),
            update.take::<bool>("extend"),
        ) {
            (Some(animes), Some(true)) => self.all_animes.extend(animes),
            (Some(animes), _) => {
                self.all_animes = animes;
                self.navigatable.back_to_start();
            }
            _ => {}
        }

        if self.bg_startup {
            self.filtered_animes = self.all_animes.clone();
        }

        if let Some(startup) = update.take::<bool>("startup") {
            self.bg_startup = startup;
        }

        if let Some(filtered_animes) = update.take::<Vec<Anime>>("filtered_animes") {
            self.filtered_animes = filtered_animes;
            self.navigatable.back_to_start();
        }
    }
}
