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
            scroll_offset: 0,
            focus: Focus::AnimeList,
            x: 0,
            y: 0,
            loading: false,
            year,
            season,
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
        let title = Paragraph::new(format!("{}: {}", self.season, self.year))
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
                if ((self.y)*3 + self.x) == index as u16 {
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

        let example_title = ["Fire Force Season 3", "Enen no Shouboutai: San no ShouEnen no Shouboutai: San no ShouEnen no Shouboutai: San no ShouEnen no Shouboutai: San no Shou"];


        let title = Paragraph::new(format!("English:\n{}\n\nJapanese:\n{}", example_title[0], example_title[1]))
            .block(Block::default().padding(Padding::new(1, 1, 1, 1)));
        frame.render_widget(title, top);

        let details = ["Type:", "Episodes:", "Status:", "Aired:", "Producers:", "Genres:", "Duration:", "Rating:", "Score:", "Ranked:", "Popularity:", "Members:", "Favorites:", "Studios"];

        if middle.width > 50 {
            let [rigth, left] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ]
                )
                .areas(middle);
            let details_left = Paragraph::new(format!("{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
                details[0], details[1], details[2], details[3], details[4], details[5],
                details[6]))
                .block(Block::default().padding(Padding::new(1, 1, 1, 1)).borders(Borders::TOP).padding(Padding::new(1, 2, 1, 1)));
            let details_right = Paragraph::new(format!("{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
                details[7], details[8], details[9], details[10], details[11], details[12],
                details[13]))
                .block(Block::default().padding(Padding::new(1, 1, 1, 1)).borders(Borders::TOP).padding(Padding::new(1, 2, 1, 1)));
            frame.render_widget(details_left, left);
            frame.render_widget(details_right, rigth);
        }
        else{
            let details = Paragraph::new(format!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}", details[0], details[1], details[2], details[3], details[4], details[5], details[6], details[7], details[8], details[9], details[10], details[11], details[12]))
                .block(Block::default().padding(Padding::new(1, 1, 1, 1)).borders(Borders::TOP).padding(Padding::new(1, 2, 1, 1)));
            frame.render_widget(details, middle);
        }


        let desc_title = Paragraph::new("\n Description:");
        frame.render_widget(desc_title, bottom);

        let description = Paragraph::new(format!("{} {}", "some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.some long description that should be wrapped to fit the screen. This is a test description for the anime, it should be long enough to test the wrapping functionality of the terminal UI.", bottom.width))
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset, 0))
            .block(Block::default().padding(Padding::new(1, 1, 0, 0)).borders(Borders::TOP).padding(Padding::new(1, 2, 1, 1)));
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
                    let update = BackgroundUpdate::new(id.clone())
                        .set("animes", animes.clone());
                    let _ = info.app_sx.send(Event::BackgroundNotice(update));
                }
                if let Some(animes) = info.mal_client.get_current_season(9, 41){
                    let update = BackgroundUpdate::new(id.clone())
                        .set("animes", animes.clone());
                    let _ = info.app_sx.send(Event::BackgroundNotice(update));
                }
                if let Some(animes) = info.mal_client.get_current_season(50, 500){
                    let update = BackgroundUpdate::new(id.clone())
                        .set("animes", animes.clone());
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

