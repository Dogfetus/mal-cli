use super::widgets::popup::SeasonPopup;
use super::{
    BackgroundUpdate, Screen, screens::*, widgets::navbar::NavBar, widgets::popup::AnimePopup,
};
use crate::screens::widgets::animebox::AnimeBox;
use crate::utils::customThreadProtocol::{CustomResizeResponse, CustomThreadProtocol};
use crate::utils::imageManager::ImageManager;
use crate::{
    app::{Action, Event},
    mal::{MalClient, models::anime::Anime},
    utils::stringManipulation::DisplayString,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Margin};
use ratatui::widgets::{Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Clear},
};
use ratatui_image::errors::Errors;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
enum LocalEvent {
    SeasonSwitch(u16, String),
    AnimeSelected,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Focus {
    Navbar,
    SeasonSelection,
    AnimeList,
    AnimeDetails,
    Popup,
}

#[derive(Clone)]
pub struct SeasonsScreen {
    animes: Vec<Anime>,
    navbar: NavBar,
    popup: AnimePopup,
    season_popup: SeasonPopup,
    focus: Focus,

    detail_scroll_x: u16,
    detail_scroll_y: u16,
    selected_anime: usize,
    scroll_offset: u16,
    x: u16,
    y: u16,

    fetching: bool,
    bg_loaded: bool,
    bg_notifier: Option<Sender<LocalEvent>>,

    year: u16,
    season: String,

    image_manager: Arc<Mutex<ImageManager>>,
}

impl SeasonsScreen {
    pub fn new() -> Self {
        let (year, season) = MalClient::current_season();
        let image_manager = Arc::new(Mutex::new(ImageManager::new()));

        Self {
            animes: Vec::new(),
            navbar: NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(SEARCH)
                .add_screen(LIST)
                .add_screen(PROFILE),
            popup: AnimePopup::new(),
            season_popup: SeasonPopup::new(),
            focus: Focus::AnimeList,

            detail_scroll_x: 0,
            detail_scroll_y: 0,
            selected_anime: 0,
            scroll_offset: 0,
            x: 0,
            y: 0,

            fetching: false,
            bg_loaded: false,
            bg_notifier: None,

            year,
            season,

            image_manager,
        }
    }

    fn get_selected_anime(&self) -> Anime {
        if let Some(anime) = self.animes.get(self.selected_anime) {
            anime.clone()
        } else {
            Anime::empty()
        }
    }

    fn fetch_season(&mut self) {
        if let Some(sender) = &self.bg_notifier {
            self.animes.clear();
            self.fetching = true;
            let event = LocalEvent::SeasonSwitch(self.year, self.season.clone());
            sender.send(event).unwrap_or_else(|e| {
                eprintln!("Failed to send season switch event: {}", e);
            });
        }
    }

    // used by the backend when fetching seasons
    fn filter_animes(animes: &[Anime], target_year: u16, target_season: &str) -> Vec<Anime> {
        animes
            .iter()
            .filter(|anime| {
                anime.start_season.year == target_year
                    && anime.start_season.season.to_lowercase() == target_season.to_lowercase()
            })
            .cloned()
            .collect()
    }
}

impl Screen for SeasonsScreen {
    fn draw(&self, frame: &mut Frame) {
        let anime = self.get_selected_anime();
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

        let [bottom_left, bottom_right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .areas(bottom);

        let [bl_top, bl_bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .areas(bottom_left);

        /* Displayes the navbar:
         * which after looks like this:
         * ╭──┬──┬──╮
         * ╰──┴──┴──╯
         * ╭─────╮╭─╮
         * ╰─────╯│ │
         * ╭─────╮│ │
         * │     ││ │
         * ╰─────╯╰─╯
         * */
        self.navbar.render(frame, top);

        /* Displayes the bottom sections:
         * which after looks like this (ish):
         * ╭──┬──┬──╮
         * ╰──┴──┴──╯
         * ╭─────┬──╮
         * ├─────┤  │
         * │     │  │
         * ╰─────┴──╯
         * */
        let (right_set, right_border) = (
            symbols::border::Set {
                bottom_right: symbols::line::ROUNDED_BOTTOM_RIGHT,
                top_right: symbols::line::ROUNDED_TOP_RIGHT,
                ..symbols::border::PLAIN
            },
            Borders::RIGHT | Borders::BOTTOM | Borders::TOP,
        );

        // bottom left top (blt)
        let (blt_set, blt_border) = (
            symbols::border::Set {
                top_left: symbols::line::ROUNDED_TOP_LEFT,
                bottom_left: symbols::line::NORMAL.vertical_right,
                top_right: symbols::line::NORMAL.horizontal_down,
                bottom_right: symbols::line::NORMAL.vertical_left,
                ..symbols::border::PLAIN
            },
            Borders::ALL,
        );

        let (blb_set, blb_border) = (
            symbols::border::Set {
                bottom_left: symbols::line::ROUNDED_BOTTOM_LEFT,
                bottom_right: symbols::line::NORMAL.horizontal_up,
                ..symbols::border::PLAIN
            },
            Borders::LEFT | Borders::BOTTOM | Borders::RIGHT,
        );

        let color = Style::default().fg(Color::DarkGray);

        frame.render_widget(
            Block::new()
                .border_set(right_set)
                .borders(right_border)
                .border_style(color),
            bottom_right,
        );
        frame.render_widget(
            Block::new()
                .border_set(blt_set)
                .borders(blt_border)
                .border_style(color),
            bl_top,
        );
        frame.render_widget(
            Block::new()
                .border_set(blb_set)
                .borders(blb_border)
                .border_style(color),
            bl_bottom,
        );

        // the season and year at the top:
        let season_color = if self.focus == Focus::SeasonSelection {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        let title = Paragraph::new(
            DisplayString::new()
                .add(&self.season)
                .add(&self.year)
                .capitalize(0)
                .build("{0} {1}"),
        )
        .centered()
        .style(Style::default().fg(season_color))
        .block(Block::default().padding(Padding::vertical(1)));
        frame.render_widget(title, bl_top);

        /* then create grid for animes:
         *  margin to keep grid from ruining area:
         * this part:
         * ╭─────╮
         * ╰─────╯
         * ╭─────╮
         * │     │
         * ╰─────╯
         */
        let [blb_top, blb_middle, blb_bottom] = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .areas(bl_bottom);

        if self.fetching && self.animes.len() < 9 {
            let [_, middle, _] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .areas(blb_middle);

            let title = Paragraph::new("Loading...")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(title, middle);
        } else {
            for (row_nr, &column) in [blb_top, blb_middle, blb_bottom].iter().enumerate() {
                let [left, middle, right] = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(34),
                    ])
                    .areas(column);
                for (column_nr, &area) in [left, middle, right].iter().enumerate() {
                    let index = (3 * (row_nr + self.scroll_offset as usize)) + column_nr;
                    let highlight = self.selected_anime == index && self.focus == Focus::AnimeList;

                    // this is each anime box:
                    let anime = if index < self.animes.len() {
                        &self.animes[index]
                    } else {
                        &Anime::empty()
                    };
                    AnimeBox::render(anime, &self.image_manager, frame, area, highlight);
                    // this is each anime box^
                }
            }
        }

        /* render right side with anime data:
         * this part:
         * ╭─╮
         * │ │
         * │ │
         * │ │
         * ╰─╯
         */

        let [bottom_right, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(1)])
            .areas(bottom_right);

        let [top, middle, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .vertical_margin(1)
            .constraints([
                Constraint::Length(7),
                Constraint::Percentage(55),
                Constraint::Percentage(45),
            ])
            .areas(bottom_right);

        let has_english_title = !anime.alternative_titles.en.is_empty();
        let title = if has_english_title {
            Paragraph::new(format!(
                "English:\n{}\n\nJapanese:\n{}",
                anime.alternative_titles.en, anime.title
            ))
        } else {
            Paragraph::new(format!(
                "English:\n{}\n\nJapanese:\n{}",
                anime.title, anime.alternative_titles.ja
            ))
        }
        .block(Block::default().padding(Padding::new(1, 1, 1, 1)));
        let genres_string = anime
            .genres
            .iter()
            .map(|g| g.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let studios_string = anime
            .studios
            .iter()
            .map(|g| g.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        frame.render_widget(title, top);

        let details = [
            ("Type:", anime.media_type),
            ("Episodes:", anime.num_episodes.to_string()),
            ("Status:", anime.status),
            ("Aired:", anime.start_date),
            ("Genres:", genres_string),
            ("Duration:", anime.average_episode_duration.to_string()),
            ("Rating:", anime.rating),
            ("Score:", anime.mean.to_string()),
            ("Ranked:", anime.rank.to_string()),
            ("Popularity:", anime.popularity.to_string()),
            ("Studios:", studios_string),
            ("Season:", anime.start_season.to_string()),
            ("Created at:", anime.created_at),
            ("Updated at:", anime.updated_at),
        ];

        fn create_details_text(details: &[(&str, String)]) -> String {
            details
                .iter()
                .map(|(label, value)| format!("{} {}", label, value))
                .collect::<Vec<_>>()
                .join("\n\n")
        }

        if middle.width > 50 {
            let [right, left] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(middle);

            let split = details.len() / 2;
            let (left_details, right_details) = details.split_at(split);
            let block_style = Block::default()
                .borders(Borders::TOP)
                .padding(Padding::new(1, 2, 1, 1));
            let details_left =
                Paragraph::new(create_details_text(left_details)).block(block_style.clone());

            let details_right =
                Paragraph::new(create_details_text(right_details)).block(block_style);

            frame.render_widget(details_left, left);
            frame.render_widget(details_right, right);
        } else {
            let details_paragraph = Paragraph::new(create_details_text(&details)).block(
                Block::default()
                    .borders(Borders::TOP)
                    .padding(Padding::new(1, 2, 1, 1)),
            );
            frame.render_widget(details_paragraph, middle);
        }

        let desc_title = Paragraph::new("\n Description:");
        let description = Paragraph::new(anime.synopsis)
            .wrap(Wrap { trim: true })
            .scroll((self.detail_scroll_y, 0))
            .block(
                Block::default()
                    .padding(Padding::new(1, 1, 0, 0))
                    .borders(Borders::TOP)
                    .padding(Padding::new(1, 2, 1, 1)),
            );

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state = ScrollbarState::new(20).position(self.detail_scroll_y as usize);

        frame.render_widget(desc_title, bottom);
        frame.render_widget(description, bottom);
        frame.render_stateful_widget(
            scrollbar,
            bottom.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );

        self.popup.render(&self.image_manager, frame);
        self.season_popup.render(frame, bl_top);
    }

    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match self.focus {
            Focus::Navbar => {
                if (key_event.code == KeyCode::Char('k') || key_event.code == KeyCode::Down)
                    && key_event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    self.navbar.deselect();
                    self.focus = Focus::SeasonSelection;
                }

                return self.navbar.handle_input(key_event);
            }
            Focus::AnimeList => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.focus = Focus::SeasonSelection;
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.focus = Focus::AnimeDetails;
                        }
                        _ => {}
                    }
                } else {
                    match key_event.code {
                        KeyCode::Up | KeyCode::Char('j') => {
                            self.y = self.y.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('k') => {
                            if self.y < self.animes.len() as u16 / 3 {
                                self.y += 1;
                            }
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            if self.x == 0 {
                                if self.y != 0 {
                                    self.y = self.y.saturating_sub(1);
                                    self.x = 2;
                                }
                            } else {
                                self.x = self.x.saturating_sub(1);
                            }
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            if self.x == 2 {
                                if self.y < self.animes.len() as u16 / 3 {
                                    self.y += 1;
                                    self.x = 0;
                                }
                            } else {
                                self.x += 1;
                            }
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if self.selected_anime < self.animes.len() {
                                self.popup.set_anime(self.get_selected_anime());
                                self.popup.toggle();
                                self.focus = Focus::Popup;
                                return None;
                            }
                        }
                        _ => {}
                    };

                    self.detail_scroll_y = 0;
                    self.detail_scroll_x = 0;
                }

                // Handle scrolling
                match self.y as i16 - self.scroll_offset as i16 {
                    3 => {
                        self.scroll_offset += 1;
                    }
                    -1 => {
                        self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    }
                    _ => {}
                }
                self.selected_anime = ((self.y * 3) + self.x) as usize;
            }

            Focus::Popup => {
                if key_event.code == KeyCode::Char('q') {
                    self.focus = Focus::AnimeList;
                }
                self.popup.handle_input(key_event);
            }

            Focus::AnimeDetails => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.navbar.select();
                            self.focus = Focus::Navbar;
                        }
                        KeyCode::Char('h') | KeyCode::Down => {
                            self.focus = Focus::AnimeList;
                        }
                        _ => {}
                    }
                } else {
                    match key_event.code {
                        KeyCode::Char('k') | KeyCode::Down => {
                            self.detail_scroll_y += 1;
                        }
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.detail_scroll_y = self.detail_scroll_y.saturating_sub(1);
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.detail_scroll_x = self.detail_scroll_x.saturating_sub(1);
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.detail_scroll_x += 1;
                        }
                        _ => {}
                    }
                }
            }

            Focus::SeasonSelection => {
                // Handle season selection input here if needed
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.navbar.select();
                            self.focus = Focus::Navbar;
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.focus = Focus::AnimeDetails;
                        }
                        KeyCode::Char('k') | KeyCode::Down => {
                            self.focus = Focus::AnimeList;
                        }
                        _ => {}
                    }
                    self.season_popup.hide();
                } else {
                    if self.season_popup.is_toggled() {
                        if let Some((year, season)) = self.season_popup.handle_input(key_event) {
                            if year == self.year && season == self.season {
                                return None; // No change, do nothing
                            }
                            self.year = year;
                            self.season = season;

                            self.fetch_season();
                        }
                    }
                    match key_event.code {
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            self.season_popup.toggle(self.year);
                        }
                        _ => {}
                    }
                }
            }
        }

        None
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
    }

    fn background(&mut self, info: super::BackgroundInfo) -> Option<std::thread::JoinHandle<()>> {
        if self.bg_loaded {
            return None;
        }
        let id = self.get_name();
        let manager = self.image_manager.clone();
        ImageManager::init_with_dedicated_thread(&manager, info.app_sx.clone(), id.clone());
        let nr_of_animes = self.animes.len();
        let (sender, receiver) = channel::<LocalEvent>();
        self.bg_loaded = true;
        self.fetching = true;
        self.bg_notifier = Some(sender);

        Some(thread::spawn(move || {
            let process_animes = |animes: Vec<Anime>, fetch_images: bool| {
                if fetch_images {
                    for anime in &animes {
                        ImageManager::fetch_image_sequential(&manager, anime);
                    }
                }
                let update = BackgroundUpdate::new(id.clone()).set("animes", animes);
                let _ = info.app_sx.send(Event::BackgroundNotice(update));
            };

            if nr_of_animes <= 0 {
                let (current_year, current_season) = MalClient::current_season();
                if let Some(animes) = info.mal_client.get_current_season(0, 20) {
                    let filtered = Self::filter_animes(&animes, current_year, &current_season);
                    process_animes(filtered, true);
                }
            }

            let update = BackgroundUpdate::new(id.clone()).set("fetching", false);
            let _ = info.app_sx.send(Event::BackgroundNotice(update));

            while let Ok(event) = receiver.recv() {
                match event {
                    LocalEvent::AnimeSelected => break,
                    LocalEvent::Stop => return,
                    LocalEvent::SeasonSwitch(year, season) => {
                        if let Some(animes) =
                            info.mal_client
                                .get_seasonal_anime(year, season.clone(), 0, 20)
                        {
                            let filtered = Self::filter_animes(&animes, year, &season);
                            process_animes(filtered, true);
                            let update = BackgroundUpdate::new(id.clone()).set("fetching", false);
                            let _ = info.app_sx.send(Event::BackgroundNotice(update));
                        }
                    }
                }
            }
        }))
    }

    fn apply_update(&mut self, update: BackgroundUpdate) {
        if let Some(animes) = update.get::<Vec<Anime>>("animes") {
            self.animes.extend(animes.iter().cloned());
        }
        if let Some(fetching) = update.get::<bool>("fetching") {
            self.fetching = *fetching;
        }
    }

    fn image_redraw(&mut self, id: usize, response: Result<CustomResizeResponse, Errors>) {
        self.image_manager
            .lock()
            .unwrap()
            .update_image(id, response);
    }
}
