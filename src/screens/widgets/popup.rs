use std::sync::{Arc, Mutex};

use crate::{
    app::Action,
    mal::{MalClient, models::anime::Anime},
    utils::imageManager::ImageManager,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{self, Color, Style},
    symbols::{self, border},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};
use std::cmp::min;

const AVAILABLE_SEASONS: [&str; 4] = ["Winter", "Spring", "Summer", "Fall"];
const FIRST_YEAR: u16 = 1917;
const FIRST_SEASON: &str = "Winter";

#[derive(Clone)]
pub struct AnimePopup {
    pub anime: Anime,
    pub toggled: bool,
    pub buttons: Vec<&'static str>,
}

impl AnimePopup {
    pub fn new() -> Self {
        Self {
            anime: Anime::empty(),
            toggled: false,
            buttons: vec!["Play", "Add to List", "Add to Favorites", "Rate", "Share"],
        }
    }

    pub fn set_anime(&mut self, anime: Anime) -> &Self {
        self.anime = anime;
        self
    }

    pub fn toggle(&mut self) -> &Self {
        self.toggled = !self.toggled;
        self
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Char('q') => {
                self.toggled = false;
                None
            }
            _ => None,
        }
    }

    pub fn render(&self, image_manager: &Arc<Mutex<ImageManager>>, frame: &mut Frame) {
        if !self.toggled {
            return;
        }
        let area = frame.area();

        let [height, width] = [area.height * 8 / 10, area.width * 7 / 10];
        let popup_area = Rect::new(
            area.x + (area.width - width) / 2,
            area.y + (area.height - height) / 2,
            width,
            height,
        );
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .style(Style::default().fg(Color::White));
        frame.render_widget(block.clone(), popup_area);

        let [_, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .areas(popup_area);
        let [_, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(right);

        let (right_set, right_border) = (
            symbols::border::Set {
                bottom_left: symbols::line::ROUNDED_BOTTOM_RIGHT,
                top_right: symbols::line::ROUNDED_BOTTOM_RIGHT,
                ..symbols::border::ROUNDED
            },
            Borders::ALL,
        );

        let right_block = Block::default()
            .borders(right_border)
            .border_set(right_set)
            .style(Style::default().fg(Color::White));

        let bottom_area = Rect::new(
            bottom.x + 1,
            bottom.y + 1,
            bottom.width - 1,
            bottom.height - 1, // Leave space for the border
        );

        frame.render_widget(Clear, bottom);
        frame.render_widget(right_block, bottom);

        let button_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Ratio(1, self.buttons.len() as u32);
                self.buttons.len()
            ])
            .split(bottom_area);

        for (button, &area) in self.buttons.iter().zip(button_areas.iter()) {
            let paragraph = Paragraph::new(button.to_string())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED),
                )
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White));
            frame.render_widget(paragraph, area);
        }
        ImageManager::render_image(
            image_manager,
            self.anime.id,
            frame,
            popup_area.inner(Margin::new(1, 1)),
        );
    }
}

#[derive(Clone)]
pub struct SeasonPopup {
    toggled: bool,
    year_scroll: u16,
    season_scroll: u16,
    year_selected: bool,
    available_years: Vec<String>,
    all_years: Vec<String>,
    entered_number: String,
}
impl SeasonPopup {
    pub fn new() -> Self {
        let (year, season) = MalClient::current_season();
        let season_scroll = AVAILABLE_SEASONS
            .iter()
            .position(|&s| s.to_lowercase() == season.to_lowercase())
            .unwrap_or(0) as u16;

        let all_years: Vec<String> = (FIRST_YEAR..=year).rev().map(|y| y.to_string()).collect();

        Self {
            toggled: false,
            year_scroll: 0,
            season_scroll,
            available_years: all_years.clone(),
            all_years,
            year_selected: false,
            entered_number: String::new(),
        }
    }

    fn filter_years(&mut self) {
        if self.entered_number.is_empty() {
            self.available_years = self.all_years.clone();
        } else {
            self.available_years = self
                .all_years
                .iter()
                .filter(|year| year.contains(&self.entered_number))
                .cloned()
                .collect();
        }
        self.year_scroll = 0;
    }

    pub fn hide(&mut self) -> &Self {
        self.toggled = false;
        self
    }

    pub fn toggle(&mut self, year: u16) -> &Self {
        self.toggled = !self.toggled;

        if self.toggled {
            self.year_scroll = self
                .available_years
                .iter()
                .position(|y| y.parse::<u16>().unwrap_or(0) == year)
                .unwrap_or(0) as u16;
        }
        self
    }

    pub fn is_toggled(&self) -> bool {
        self.toggled
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<(u16, String)> {
        match key_event.code {
            KeyCode::Char('q') => {
                self.toggled = false;
                self.entered_number.clear();
                self.filter_years();
                None
            }

            KeyCode::Right | KeyCode::Char('l') => {
                self.year_selected = false;
                None
            }

            KeyCode::Left | KeyCode::Char('h') => {
                self.year_selected = true;
                None
            }

            KeyCode::Up | KeyCode::Char('j') => {
                if self.year_selected {
                    if self.year_scroll > 0 {
                        self.year_scroll -= 1;
                    }
                } else {
                    if self.season_scroll > 0 {
                        self.season_scroll -= 1;
                    }
                }
                None
            }
            KeyCode::Down | KeyCode::Char('k') => {
                if self.year_selected {
                    if self.year_scroll < (self.available_years.len() - 1) as u16 {
                        self.year_scroll += 1;
                    }
                } else {
                    if self.season_scroll < (AVAILABLE_SEASONS.len() - 1) as u16 {
                        self.season_scroll += 1;
                    }
                }
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let (year, _) = MalClient::current_season();
                let season = AVAILABLE_SEASONS
                    .get(self.season_scroll as usize)
                    .unwrap_or(&FIRST_SEASON)
                    .to_string();

                let year = self
                    .available_years
                    .get(self.year_scroll as usize)
                    .and_then(|y| y.parse::<u16>().ok())
                    .unwrap_or(year);

                self.entered_number.clear();
                self.filter_years();
                Some((year, season))
            }
            KeyCode::Backspace => {
                if !self.entered_number.is_empty() {
                    self.entered_number.pop();
                    self.filter_years();
                }
                None
            }
            KeyCode::Char(c) => {
                if c.is_digit(10) {
                    self.entered_number.push(c);
                    self.filter_years();
                }
                None
            }
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame, season_area: Rect) {
        if !self.toggled {
            return;
        }
        let area = frame.area();

        let [height, width] = [min(8, area.height), season_area.width * 7 / 20];
        let popup_area = Rect::new(
            season_area.x + (season_area.width - width) / 2,
            season_area.y + season_area.height - 1,
            width,
            height,
        );
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .style(Style::default().fg(Color::White));
        frame.render_widget(block.clone(), popup_area);

        let text = if self.entered_number.is_empty() {
            self.entered_number.clone()
        } else {
            format!("Search: {}", self.entered_number)
        };
        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        frame.render_widget(paragraph, popup_area);

        let [year_area, middle_area, season_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ])
            .areas(popup_area);
        let season_area = Rect {
            x: season_area.x + 1,
            y: season_area.y + 1,
            width: season_area.width - 2,
            height: season_area.height - 2,
        };
        let year_area = Rect {
            x: year_area.x + 1,
            y: year_area.y + 1,
            width: year_area.width - 2,
            height: year_area.height - 2,
        };

        let divider = "|";
        let left_arrow = if self.year_selected { "◀" } else { " " };
        let right_arrow = if !self.year_selected { "▶" } else { " " };
        let [middle_area_left, middle_area, middle_area_right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(middle_area);

        let middle_area = Rect {
            x: middle_area.x,
            y: middle_area.y + 2,
            width: middle_area.width,
            height: middle_area.height - 3,
        };

        let middle_area_left = Rect {
            x: middle_area_left.x,
            y: middle_area_left.y + 2,
            width: middle_area_left.width,
            height: middle_area_left.height - 3,
        };

        let middle_area_right = Rect {
            x: middle_area_right.x,
            y: middle_area_right.y + 2,
            width: middle_area_right.width,
            height: middle_area_right.height - 3,
        };

        let left_paragraph = Paragraph::new(left_arrow)
            .block(Block::default().padding(Padding::new(0, 0, middle_area_left.height / 2, 0)))
            .alignment(Alignment::Left)
            .style(Style::default().fg(if self.year_selected {
                Color::Yellow
            } else {
                Color::White
            }));
        let middle_paragraph = Paragraph::new(divider)
            .block(Block::default().padding(Padding::new(0, 0, middle_area.height / 2, 0)))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        let right_paragraph = Paragraph::new(right_arrow)
            .block(Block::default().padding(Padding::new(0, 0, middle_area_right.height / 2, 0)))
            .alignment(Alignment::Right)
            .style(Style::default().fg(if !self.year_selected {
                Color::Yellow
            } else {
                Color::White
            }));

        frame.render_widget(left_paragraph, middle_area_left);
        frame.render_widget(middle_paragraph, middle_area);
        frame.render_widget(right_paragraph, middle_area_right);

        for (i, season) in AVAILABLE_SEASONS.iter().enumerate() {
            let y_position = (3 + season_area.y + i as u16).saturating_sub(self.season_scroll);
            if y_position >= season_area.y + season_area.height {
                break;
            }
            let individual_season_area = Rect {
                x: season_area.x,
                y: y_position,
                width: season_area.width,
                height: 1,
            };
            let paragraph = Paragraph::new(season.to_string())
                .alignment(Alignment::Center)
                .style(Style::default().fg(
                    if !self.year_selected && self.season_scroll == i as u16 {
                        Color::Yellow
                    } else {
                        Color::White
                    },
                ));
            frame.render_widget(paragraph, individual_season_area);
        }

        for (i, year) in self.available_years.iter().enumerate() {
            let y_position = (3 + year_area.y + i as u16).saturating_sub(self.year_scroll);
            if y_position >= year_area.y + year_area.height {
                break;
            } else if y_position < year_area.y {
                continue;
            }
            let individual_year_area = Rect {
                x: year_area.x,
                y: y_position,
                width: year_area.width,
                height: 1,
            };
            let paragraph = Paragraph::new(year.to_string())
                .alignment(Alignment::Center)
                .style(Style::default().fg(
                    if self.year_selected && self.year_scroll == i as u16 {
                        Color::Yellow
                    } else {
                        Color::White
                    },
                ));
            frame.render_widget(paragraph, individual_year_area);
        }
    }
}

#[derive(Clone)]
pub struct SelectionPopup {
    pub is_open: bool,
    pub options: Vec<&'static str>,
    pub selected_index: usize,
}

impl SelectionPopup {
    pub fn new() -> Self {
        Self {
            is_open: false,
            options: Vec::new(),
            selected_index: 0,
        }
    }

    pub fn add_option(mut self, option: &'static str) -> Self {
        self.options.push(option);
        self
    }

    pub fn open(&mut self) -> &Self {
        self.is_open = true;
        self.selected_index = 0;
        self
    }

    pub fn close(&mut self) -> &Self {
        self.is_open = false;
        self.selected_index = 0;
        self
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        if !self.is_open {
            match key_event.code{
                KeyCode::Enter => {
                    self.open();
                }
                _ => {},
            }
            None
        }
        else{
            match key_event.code {
                KeyCode::Char('q') => {
                    self.is_open = false;
                    None
                }
                KeyCode::Up | KeyCode::Char('j') => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    None
                }
                KeyCode::Down | KeyCode::Char('k') => {
                    if self.selected_index < self.options.len() - 1 {
                        self.selected_index += 1;
                    }
                    None
                }
                _ => None,
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, highlighted: bool) {
        let filter = Paragraph::new(
            *self.options.get(self.selected_index).unwrap_or(&"No options")
            ).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(border::ROUNDED),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(if highlighted {
                Color::Yellow
            } else {
                Color::DarkGray
            }));
        frame.render_widget(filter, area);

        if self.is_open {
            let options_area = Rect::new(
                area.x,
                area.y + area.height,
                area.width,
                self.options.len() as u16 + 2,
            );

            frame.render_widget(Clear, options_area);

            let options_block = Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .style(Style::default().fg(Color::DarkGray));

            frame.render_widget(options_block, options_area);

            for (i, option) in self.options.iter().enumerate() {
                let option_area = Rect::new(
                    options_area.x,
                    options_area.y + i as u16 + 1,
                    options_area.width-1,
                    1,
                );
                let option_paragraph = Paragraph::new(option.to_string())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(if highlighted && i == self.selected_index {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    }));
                frame.render_widget(option_paragraph, option_area);
            }
        }
    }
}

#[derive(Clone)]
pub struct SearchPopup {
    pub toggled: bool,
    pub query: String,
}
