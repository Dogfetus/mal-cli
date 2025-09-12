use super::ExtraInfo;
use super::widgets::navigatable::Navigatable;
use super::widgets::popup::SeasonPopup;
use super::{BackgroundUpdate, Screen};
use crate::add_screen_caching;
use crate::config::TEXT_COLOR;
use crate::mal::models::anime::AnimeId;
use crate::{
    app::{Action, Event},
    config::{HIGHLIGHT_COLOR, PRIMARY_COLOR},
    mal::{MalClient, models::anime::Anime},
    screens::widgets::animebox::AnimeBox,
    utils::{
        functionStreaming::StreamableRunner, imageManager::ImageManager,
        stringManipulation::DisplayString,
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Margin, Rect};
use ratatui::widgets::{Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    symbols,
    widgets::{Block, Borders, Clear},
};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

// TODO: remember to fetch all season anime

#[derive(Debug, Clone)]
enum LocalEvent {
    SeasonSwitch(u16, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Focus {
    Navbar,
    SeasonSelection,
    AnimeList,
    AnimeDetails,
}

#[derive(Clone)]
pub struct SeasonsScreen {
    animes: Vec<AnimeId>,
    season_popup: SeasonPopup,
    focus: Focus,
    app_info: ExtraInfo,

    detail_scroll_x: u16,
    detail_scroll_y: u16,

    fetching: bool,
    bg_loaded: bool,
    bg_notifier: Option<Sender<LocalEvent>>,

    year: u16,
    season: String,

    image_manager: Arc<Mutex<ImageManager>>,
    navigatable: Navigatable,
}

impl SeasonsScreen {
    pub fn new(info: ExtraInfo) -> Self {
        let (year, season) = MalClient::current_season();
        let image_manager = Arc::new(Mutex::new(ImageManager::new()));

        Self {
            animes: Vec::new(),
            season_popup: SeasonPopup::new(),
            focus: Focus::AnimeList,
            app_info: info,

            detail_scroll_x: 0,
            detail_scroll_y: 0,

            fetching: false,
            bg_loaded: false,
            bg_notifier: None,

            year,
            season,

            image_manager,
            navigatable: Navigatable::new((3, 3)),
        }
    }

    fn fetch_season(&mut self) {
        if let Some(sender) = &self.bg_notifier {
            self.animes.clear();
            self.fetching = true;
            self.navigatable.back_to_start();
            let event = LocalEvent::SeasonSwitch(self.year, self.season.clone());
            sender.send(event).unwrap_or_else(|e| {
                eprintln!("Failed to send season switch event: {}", e);
            });
        }
    }

    // apperently the animes gotten include previous seasons that has not yet finished
    fn filter_animes(animes: Vec<Anime>, target_year: u16, target_season: &String) -> Vec<Anime> {
        animes
            .iter()
            .filter(|anime| {
                anime.start_season.year == target_year
                    && anime.start_season.season.to_lowercase() == target_season.to_lowercase()
            })
            .cloned()
            .collect()
    }

    fn fetch_anime_season(
        year: u16,
        season: String,
        app_sx: &Sender<Event>,
        mal_client: &Arc<MalClient>,
        id: String,
    ) {
        let anime_batches = StreamableRunner::new()
            .change_batch_size_at(500, 1)
            .stop_early();

        for batch in anime_batches
            .run(|offset, limit| mal_client.get_seasonal_anime(year, season.clone(), offset, limit))
        {
            let animes = Self::filter_animes(batch, year, &season);
            let anime_ids = animes.iter().map(|a| a.id).collect::<Vec<_>>();

            let update = BackgroundUpdate::new(id.clone())
                .set("animes", animes)
                .set("anime_ids", anime_ids)
                .set("fetching", false)
                .set("extend", true);
            app_sx.send(Event::BackgroundNotice(update)).ok();
        }
    }
}

impl Screen for SeasonsScreen {
    add_screen_caching!();

    fn draw(&mut self, frame: &mut Frame) {
        let mut anime = Anime::empty();
        if let Some(selected_anime) = self.navigatable.get_selected_item(&self.animes) {
            if let Some(found_anime) = self.app_info.anime_store.get(selected_anime) {
                anime = (*found_anime).clone();
            }
        }

        let area = frame.area();
        frame.render_widget(Clear, area);

        /* Splitting the screen:
         * which after looks like this:
         * ╭──┬──┬──╮
         * ╰──┴──┴──╯
         * ╭─────╮╭─╮
         * ╰─────╯│ │
         * ╭─────╮│ │
         * │     ││ │
         * ╰─────╯╰─╯
         * */
        let [_, bottom] = Layout::default()
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
                bottom_left: symbols::line::ROUNDED_BOTTOM_LEFT,
                top_right: symbols::line::NORMAL.horizontal_down,
                bottom_right: symbols::line::NORMAL.vertical_left,
                ..symbols::border::PLAIN
            },
            Borders::ALL,
        );

        let (blb_set, blb_border) = (
            symbols::border::Set {
                horizontal_bottom: " ".into(),
                bottom_right: symbols::line::ROUNDED_BOTTOM_LEFT,
                ..symbols::border::PLAIN
            },
            Borders::RIGHT | Borders::BOTTOM,
        );

        let color = Style::default().fg(PRIMARY_COLOR);

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
            HIGHLIGHT_COLOR
        } else {
           TEXT_COLOR 
        };
        let title = Paragraph::new(
            DisplayString::new()
                .add(&self.season)
                .add(self.year)
                .add(self.animes.len())
                .capitalize(0)
                .build("{0} {1} ({2})"),
        )
        .centered()
        .style(Style::default().fg(season_color));
        frame.render_widget(title, bl_top.inner(Margin::new(5, 1)));

        /* then create grid for animes:
         *  margin to keep grid from ruining area:
         * this part:
         * ╭─────╮
         * ╰─────╯
         * ╭─────╮
         * │     │
         * ╰─────╯
         */
        if self.animes.is_empty() && self.fetching {
            let [_, middle, _] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .areas(bl_bottom);
            let title = Paragraph::new("Loading...")
                .alignment(Alignment::Center)
                .style(Style::default().fg(PRIMARY_COLOR));
            frame.render_widget(
                title,
                middle.inner(Margin {
                    vertical: 0,
                    horizontal: 1,
                }),
            );
        } else {
            let area = Rect::new(
                bl_bottom.x,
                bl_bottom.y,
                bl_bottom.width.saturating_sub(2),
                bl_bottom.height,
            );
            self.navigatable
                .construct(&self.animes, area, |anime_id, area, highlight| {
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
            .style(Style::default().fg(TEXT_COLOR))
        } else {
            Paragraph::new(format!(
                "English:\n{}\n\nJapanese:\n{}",
                anime.title, anime.alternative_titles.ja
            ))
            .style(Style::default().fg(TEXT_COLOR))
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
                .border_style(Style::default().fg(PRIMARY_COLOR))
                .padding(Padding::new(1, 2, 1, 1));
            let details_left =
                Paragraph::new(create_details_text(left_details))
                .style(Style::default().fg(TEXT_COLOR))
                .block(block_style.clone());

            let details_right =
                Paragraph::new(create_details_text(right_details))
                .style(Style::default().fg(TEXT_COLOR))
                .block(block_style);

            frame.render_widget(details_left, left);
            frame.render_widget(details_right, right);
        } else {
            let details_paragraph = Paragraph::new(create_details_text(&details))
                .style(Style::default().fg(TEXT_COLOR))
                .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(PRIMARY_COLOR))
                    .padding(Padding::new(1, 2, 1, 1)),
            );
            frame.render_widget(details_paragraph, middle);
        }

        let desc_title = Paragraph::new("\n Description:");
        let description = Paragraph::new(anime.synopsis)
            .style(Style::default().fg(TEXT_COLOR))
            .wrap(Wrap { trim: true })
            .scroll((self.detail_scroll_y, 0))
            .block(
                Block::default()
                    .padding(Padding::new(1, 1, 0, 0))
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(PRIMARY_COLOR))
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

        self.season_popup.render(frame, bl_top);
    }

    fn handle_keyboard(&mut self, key_event: KeyEvent) -> Option<Action> {
        match self.focus {
            // this focus is used just to not highligh anything in the screen
            // and when the navbar gets deselcted this handle_input will run once right after
            // whcih will set its focus back to the seasonselection
            Focus::Navbar => {
                self.focus = Focus::SeasonSelection;
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
                            self.navigatable.move_up();
                        }
                        KeyCode::Down | KeyCode::Char('k') => {
                            self.navigatable.move_down();
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            self.navigatable.move_left();
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            self.navigatable.move_right();
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if let Some(id) = self.navigatable.get_selected_item(&self.animes) {
                                if let Some(anime) = self.app_info.anime_store.get(id) {
                                    return Some(Action::ShowOverlay(anime.id));
                                }
                            }
                        }
                        _ => {}
                    };

                    self.detail_scroll_y = 0;
                    self.detail_scroll_x = 0;
                }
            }

            Focus::AnimeDetails => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.focus = Focus::Navbar;
                            return Some(Action::NavbarSelect(true));
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
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
                            self.focus = Focus::Navbar;
                            return Some(Action::NavbarSelect(true));
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

    fn background(&mut self) -> Option<std::thread::JoinHandle<()>> {
        if self.bg_loaded {
            return None;
        }

        let (sender, receiver) = channel::<LocalEvent>();
        self.bg_loaded = true;
        self.fetching = true;
        self.bg_notifier = Some(sender);

        let id = self.get_name();
        let info = self.app_info.clone();
        let nr_of_animes = self.animes.len();
        let manager = self.image_manager.clone();

        ImageManager::init_with_threads(&manager, info.app_sx.clone());

        Some(thread::spawn(move || {
            if nr_of_animes <= 0 {
                let (year, season) = MalClient::current_season();
                Self::fetch_anime_season(year, season, &info.app_sx, &info.mal_client, id.clone());
            }

            while let Ok(event) = receiver.recv() {
                match event {
                    LocalEvent::SeasonSwitch(year, season) => {
                        Self::fetch_anime_season(
                            year,
                            season,
                            &info.app_sx,
                            &info.mal_client,
                            id.clone(),
                        );
                    }
                }
            }
        }))
    }

    fn apply_update(&mut self, mut update: BackgroundUpdate) {
        if let Some(animes) = update.take::<Vec<AnimeId>>("anime_ids") {
            self.animes.extend(animes);
        }
        if let Some(fetching) = update.take::<bool>("fetching") {
            self.fetching = fetching;
        }
    }
}
