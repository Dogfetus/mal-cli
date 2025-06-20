use std::thread;

use super::widgets::navbar::NavBar;
use super::{screens::*, BackgroundUpdate, Screen};
use crate::mal::{models::anime::Anime, MalClient};
use crate::app::{Action, Event};
use ratatui::layout::{Alignment, Margin};
use ratatui::widgets::{Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::{
    layout::{Constraint, Direction, Layout}, style::{Color, Style}, widgets::{Block,  Borders, Clear}, Frame,
    symbols,
};
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone)]
enum Focus {
    Navbar,
    AnimeList,
}


#[derive(Clone)]
pub struct SeasonsScreen { 
    selected_anime: usize,
    animes: Vec<Anime>,
    focus: Focus,
    x: u16,
    y: u16,
    scroll_offset: u16,
    navbar: NavBar,
    loading: bool,
    year: u16,
    season: String,
}

impl SeasonsScreen {
    pub fn new() -> Self {
        let (year, season) = MalClient::current_season();

        Self {
            animes: Vec::new(),
            navbar: NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(LIST)
                .add_screen(FILTER)
                .add_screen(PROFILE),
            focus: Focus::AnimeList,
            selected_anime: 0,
            scroll_offset: 0,
            loading: false,
            x: 0,
            y: 0,
            year,
            season,
        }
    }

    fn get_selected_anime(&self) -> Anime{
        if let Some(anime) = self.animes.get(self.selected_anime){
            anime.clone()
        }
        else{
            Anime::empty()
        }
    }
}

impl Screen for SeasonsScreen {
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
            .constraints(
                [
                    Constraint::Length(4),
                    Constraint::Percentage(100),
                ]
            )
            .areas(area);


        let [bottom_left, bottom_right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ]
            )
            .areas(bottom);

        let [bl_top, bl_bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Percentage(100),
                ]
            )
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
        let (right_set, right_border) = 
        (
            symbols::border::Set {
                bottom_right: symbols::line::ROUNDED_BOTTOM_RIGHT,
                top_right: symbols::line::ROUNDED_TOP_RIGHT,
                ..symbols::border::PLAIN
            },
            Borders::RIGHT | Borders::BOTTOM | Borders::TOP
        );

        // bottom left top (blt)
        let (blt_set, blt_border) = 
        (
            symbols::border::Set {
                top_left: symbols::line::ROUNDED_TOP_LEFT,
                bottom_left: symbols::line::NORMAL.vertical_right,
                top_right: symbols::line::NORMAL.horizontal_down,
                bottom_right: symbols::line::NORMAL.vertical_left,
                ..symbols::border::PLAIN
            },
            Borders::ALL
        );

        let (blb_set, blb_border) = 
        (
            symbols::border::Set {
                bottom_left: symbols::line::ROUNDED_BOTTOM_LEFT,
                bottom_right: symbols::line::NORMAL.horizontal_up,
                ..symbols::border::PLAIN
            },
            Borders::LEFT | Borders::BOTTOM | Borders::RIGHT
        );


        let color = Style::default().fg(Color::DarkGray);

        frame.render_widget(Block::new().border_set(right_set).borders(right_border).border_style(color), bottom_right);
        frame.render_widget(Block::new().border_set(blt_set).borders(blt_border).border_style(color), bl_top);
        frame.render_widget(Block::new().border_set(blb_set).borders(blb_border).border_style(color), bl_bottom);

        // try add some ttext to bl_top: vertical center
        let title = Paragraph::new(format!("{} {}", format!("{}{}", self.season[0..1].to_uppercase(), &self.season[1..]), self.year))
            .centered()
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
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ]
            )
            .areas(bl_bottom);
        for (row_nr, &column) in [blb_top, blb_middle, blb_bottom].iter().enumerate() {
            let [left, middle, right] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ]
                )
                .areas(column);
            for (column_nr, &area) in [left, middle, right].iter().enumerate() {
                let index = (3 * (row_nr + self.scroll_offset as usize)) + column_nr;

                let title_text = self.animes
                    .get(index)
                    .map(|anime| anime.title.clone())
                    .unwrap_or("Loading...".to_string());

                let mut color = Color::Gray; 
                if self.selected_anime == index {
                    color = Color::Yellow;
                } 
                // the anime box should go here:
                let title = Paragraph::new(title_text)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(color))
                    .block(Block::default().padding(Padding::new(1, 1, 1, 1)));
                frame.render_widget(title, area);
                frame.render_widget(Block::new().borders(Borders::ALL).border_style(color), area);
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
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ]
            )
            .areas(bottom_right);

        let [top, middle, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .vertical_margin(1)
            .constraints(
                [
                    Constraint::Length(7),
                    Constraint::Percentage(55),
                    Constraint::Percentage(45),
                ]
            )
            .areas(bottom_right);

        let anime = self.get_selected_anime();
        let title = Paragraph::new(
            format!("English:\n{}\n\nJapanese:\n{}", 
                anime.title, 
                anime.alternative_titles.ja)
            )
            .block(Block::default().padding(Padding::new(1, 1, 1, 1)));
        let genres_string = anime.genres.iter()
            .map(|g| g.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let studios_string = anime.studios.iter()
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
            details.iter()
                .map(|(label, value)| format!("{} {}", label, value))
                .collect::<Vec<_>>()
                .join("\n\n")
        }

        if middle.width > 50 {
            let [right, left] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ]
                )
                .areas(middle);

            let split = details.len() / 2;
            let (left_details, right_details) = details.split_at(split);

            let block_style = Block::default()
                .borders(Borders::TOP)
                .padding(Padding::new(1, 2, 1, 1));

            let details_left = Paragraph::new(create_details_text(left_details))
                .block(block_style.clone());

            let details_right = Paragraph::new(create_details_text(right_details))
                .block(block_style);

            frame.render_widget(details_left, left);
            frame.render_widget(details_right, right);
        }
        else{
             let details_paragraph = Paragraph::new(create_details_text(&details))
                .block(Block::default()
                    .borders(Borders::TOP)
                    .padding(Padding::new(1, 2, 1, 1)));
            frame.render_widget(details_paragraph, middle);
        }


        let desc_title = Paragraph::new("\n Description:");
        frame.render_widget(desc_title, bottom);

        let description = Paragraph::new(anime.synopsis)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset, 0))
            .block(Block::default()
                .padding(Padding::new(1, 1, 0, 0))
                .borders(Borders::TOP)
                .padding(Padding::new(1, 2, 1, 1)));
        frame.render_widget(description, bottom);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state = ScrollbarState::new(20).position(self.scroll_offset as usize);
        frame.render_stateful_widget(
    scrollbar,
    bottom.inner(Margin {
            // using an inner vertical margin of 1 unit makes the scrollbar inside the block
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    }

    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        if self.navbar.is_selected() {
            self.focus = Focus::Navbar;
        } else {
            self.focus = Focus::AnimeList;
        }

        match self.focus {
            Focus::Navbar => {
                if let Some(action) = self.navbar.handle_input(key_event) {
                    return Some(action);
                }
            }
            Focus::AnimeList => {
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
                        }
                        else {
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
                    KeyCode::Enter => {
                        self.navbar.select();
                    }
                    _ => {} 
                };

                match self.y as i16 - self.scroll_offset as i16 {
                    3 => {
                        self.scroll_offset += 1;
                    }
                    -1 => {
                        self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    }
                    _ => {
                    }
                }
                self.selected_anime = ((self.y * 3) + self.x) as usize;
            }
        }

        None
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
    }

    fn background(&mut self, info: super::BackgroundInfo) -> Option<std::thread::JoinHandle<()>> {
        if self.loading {
            return None;
        }
        self.loading = true;

        let nr_of_animes = self.animes.len();
        let id = self.get_name(); 
        Some(thread::spawn(move || {
            if nr_of_animes <= 0 {
                //temporary
                if let Some(animes) = info.mal_client.get_current_season(0, 9){
                    let mut animes_to_send = Vec::<Anime>::new();
                    let (current_year, current_season) = MalClient::current_season();
                    for anime in animes.iter() {
                        if anime.start_season.year == current_year &&
                            anime.start_season.season == current_season {
                            animes_to_send.push(anime.clone());
                        }

                    }
                    let update = BackgroundUpdate::new(id.clone())
                        .set("animes", animes_to_send.clone());
                    let _ = info.app_sx.send(Event::BackgroundNotice(update));
                }
                if let Some(animes) = info.mal_client.get_current_season(9, 500){
                    let mut animes_to_send = Vec::<Anime>::new();
                    let (current_year, current_season) = MalClient::current_season();
                    for anime in animes.iter() {
                        if anime.start_season.year == current_year &&
                            anime.start_season.season == current_season {
                            animes_to_send.push(anime.clone());
                        }

                    }
                    let update = BackgroundUpdate::new(id.clone())
                        .set("animes", animes_to_send.clone());
                    let _ = info.app_sx.send(Event::BackgroundNotice(update));
                }
            }
        }))
    }

    fn apply_update(&mut self, update: BackgroundUpdate) {
        if let Some(animes) = update.get::<Vec<Anime>>("animes") {
            self.animes.extend(animes.iter().cloned());
        }
    }
}

