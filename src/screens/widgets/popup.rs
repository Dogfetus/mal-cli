use std::sync::{Arc, Mutex, mpsc::Sender};

use crate::{
    app::{Action, Event},
    config::{HIGHLIGHT_COLOR, PRIMARY_COLOR, SECONDARY_COLOR},
    mal::{MalClient, models::anime::Anime},
    utils::imageManager::ImageManager,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect}, style::{Modifier, Style, Stylize}, symbols::{self, border}, widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap}, Frame
};
use tui_widgets::big_text::{BigText, PixelSize};
use std::cmp::min;

use super::navigatable::Navigatable;

const AVAILABLE_SEASONS: [&str; 4] = ["Winter", "Spring", "Summer", "Fall"];
const FIRST_YEAR: u16 = 1917;
const FIRST_SEASON: &str = "Winter";
const BUTTON_HEIGHT: u16 = 3;

#[derive(Clone)]
pub struct AnimePopup {
    anime: Anime,
    toggled: bool,
    buttons: Vec<&'static str>,
    button_nav: Navigatable,
    image_manager: Arc<Mutex<ImageManager>>,
}

impl AnimePopup {
    pub fn new(sender: Sender<Event>) -> Self {
        let buttons = vec!["Play", "Add to List", "Favorite",  "Rate", "Share"];
        let image_manager = Arc::new(Mutex::new(ImageManager::new()));
        ImageManager::init_with_threads(&image_manager, sender.clone());

        Self {
            image_manager,
            anime: Anime::empty(),
            toggled: false,
            button_nav: Navigatable::new((buttons.len() as u16, 1)),
            buttons,
        }
    }

    pub fn set_anime(&mut self, anime: Anime) -> &Self {
        self.anime = anime;
        self
    }

    pub fn is_open(&self) -> bool {
        self.toggled
    }

    pub fn open(&mut self) -> &Self {
        self.toggled = true;
        self
    }

    pub fn close(&mut self) -> &Self {
        self.toggled = false;
        self
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Char('q') => {
                self.close();
                None
            }
            KeyCode::Char('k') | KeyCode::Down => {
                self.button_nav.move_down();
                None
            }
            KeyCode::Char('j') | KeyCode::Up => {
                self.button_nav.move_up();
                None
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.button_nav.move_right();
                None
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.button_nav.move_left();
                None
            }
            _ => None,
        }

    }

    pub fn render(&mut self, frame: &mut Frame) {
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

        // clear the space for the popup
        frame.render_widget(Clear, popup_area);

        // craete the border arond the whole popup
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .style(Style::default().fg(SECONDARY_COLOR));
        frame.render_widget(block, popup_area);

        // split the popup up so we can get the area for the bottons ont he right side
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Percentage(30)])
            .areas(popup_area);
        //buttons area
        let [_, bottom_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(self.buttons.len() as u16 * BUTTON_HEIGHT + 1),
            ])
            .areas(right);

        // now create borders that makes the top and left connect to the rest
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
            .style(Style::default().fg(SECONDARY_COLOR));
        let buttons_area = Rect::new(
            bottom_area.x + 1,
            bottom_area.y + 1,
            bottom_area.width - 1,
            bottom_area.height - 1,
        );
        frame.render_widget(right_block, bottom_area);

        // add the buttons
        self.button_nav
            .construct(&self.buttons, buttons_area, |button, area, highlighted| {
                let button_paragraph = Paragraph::new(button.to_string())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_set(border::ROUNDED),
                    )
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(if highlighted {
                        HIGHLIGHT_COLOR
                    } else {
                        SECONDARY_COLOR
                    }));
                frame.render_widget(button_paragraph, area);
            });



        // the rest of the popup

        let [title_area, info_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .areas(popup_area.inner(Margin::new(1, 1)));
        let [_, title_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Fill(1)])
            .areas(title_area);
        let info_area = Rect {
            x: title_area.x,
            y: info_area.y,
            width: title_area.width,
            height: info_area.height,
        };

        let title = if self.anime.title.is_empty() {
           self.anime.alternative_titles.en.clone()
        } else {
            self.anime.title.clone()
        };
        let title_text = Paragraph::new(title)
            .alignment(Alignment::Center)
            .style(Style::default().fg(SECONDARY_COLOR).bold());

        frame.render_widget(title_text, title_area.inner(Margin::new(0, 1)));

        let image_area = Rect {
            x: popup_area.x + 4,
            y: popup_area.y + 2,
            width: popup_area.width,
            height: bottom_area.y - popup_area.y - 3 ,
        };

        ImageManager::render_image(
            &self.image_manager,
            &self.anime,
            frame,
            image_area,
            true,
        );

        let synopsis_area = Rect {
            x: left.x + 1,
            y: bottom_area.y,
            width: left.width - 1,
            height: bottom_area.height - 1,
        };
        let synopsis_text = Paragraph::new(self.anime.synopsis.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(border::ROUNDED)
                    .title("Synopsis")
                    .style(Style::default().fg(PRIMARY_COLOR)),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(PRIMARY_COLOR));
        frame.render_widget(synopsis_text, synopsis_area);

        let big_text = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .lines(vec![
                self.anime.mean.to_string().into(),
            ])
            .build();
        frame.render_widget(big_text, info_area);

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
            .style(Style::default().fg(PRIMARY_COLOR));
        frame.render_widget(block.clone(), popup_area);

        let text = if self.entered_number.is_empty() {
            self.entered_number.clone()
        } else {
            format!("Search: {}", self.entered_number)
        };
        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(PRIMARY_COLOR));
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
                HIGHLIGHT_COLOR
            } else {
                PRIMARY_COLOR
            }));
        let middle_paragraph = Paragraph::new(divider)
            .block(Block::default().padding(Padding::new(0, 0, middle_area.height / 2, 0)))
            .alignment(Alignment::Center)
            .style(Style::default().fg(PRIMARY_COLOR));
        let right_paragraph = Paragraph::new(right_arrow)
            .block(Block::default().padding(Padding::new(0, 0, middle_area_right.height / 2, 0)))
            .alignment(Alignment::Right)
            .style(Style::default().fg(if !self.year_selected {
                HIGHLIGHT_COLOR
            } else {
                PRIMARY_COLOR
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
                        HIGHLIGHT_COLOR
                    } else {
                        PRIMARY_COLOR
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
                        HIGHLIGHT_COLOR
                    } else {
                        PRIMARY_COLOR
                    },
                ));
            frame.render_widget(paragraph, individual_year_area);
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Arrows {
    None,
    Static,
    Dynamic,
}

#[derive(Clone)]
pub struct SelectionPopup {
    is_open: bool,
    options: Vec<String>,
    selected_index: usize,
    next_index: usize,
    arrows: Arrows,
    longest_word: usize,
}

impl SelectionPopup {
    pub fn new() -> Self {
        Self {
            is_open: false,
            options: Vec::new(),
            selected_index: 0,
            next_index: 0,
            arrows: Arrows::None,
            longest_word: 0,
        }
    }

    pub fn with_arrows(mut self, arrow_type: Arrows) -> Self {
        self.arrows = arrow_type;
        self
    }

    pub fn add_option(mut self, option: impl Into<String>) -> Self {
        let option = option.into();
        if option.len() > self.longest_word {
            self.longest_word = option.len();
        }
        self.options.push(option);
        self
    }

    pub fn open(&mut self) -> &Self {
        self.is_open = true;
        self
    }

    pub fn close(&mut self) -> &Self {
        self.is_open = false;
        self
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<String> {
        if !self.is_open {
            match key_event.code {
                KeyCode::Enter => {
                    self.open();
                }
                _ => {}
            }
            None
        } else {
            match key_event.code {
                KeyCode::Char('q') => {
                    self.is_open = false;
                    None
                }
                KeyCode::Up | KeyCode::Char('j') => {
                    if self.next_index > 0 {
                        self.next_index -= 1;
                    }
                    None
                }
                KeyCode::Down | KeyCode::Char('k') => {
                    if self.next_index < self.options.len() - 1 {
                        self.next_index += 1;
                    }
                    None
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if self.options.is_empty() {
                        return None;
                    }

                    let selected_option = self.options[self.next_index].clone();
                    self.selected_index = self.next_index;
                    self.close();
                    Some(selected_option)
                }
                _ => None,
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, highlighted: bool) {
        let filter = Paragraph::new(
            self.options
                .get(self.selected_index)
                .unwrap_or(&"No options available".to_string())
                .clone(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(if highlighted {
            HIGHLIGHT_COLOR
        } else {
            PRIMARY_COLOR
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
                .style(Style::default().fg(PRIMARY_COLOR));

            frame.render_widget(options_block, options_area);

            for (i, option) in self.options.iter().enumerate() {
                let option_area = Rect::new(
                    options_area.x + 1,
                    options_area.y + i as u16 + 1,
                    options_area.width.saturating_sub(2),
                    1,
                );

                let [left_side, option_area, right_side] = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Fill(1),
                        Constraint::Length(min(
                            self.longest_word as u16 + 2,
                            option_area.width.saturating_sub(2),
                        )),
                        Constraint::Fill(1),
                    ])
                    .areas(option_area);

                if i == self.next_index {
                    let mut text = option.to_string();

                    if self.arrows == Arrows::Dynamic {
                        text = format!("▶ {} ◀", option.to_string());
                    }

                    let option_paragraph = Paragraph::new(text)
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(HIGHLIGHT_COLOR));
                    frame.render_widget(option_paragraph, option_area);

                    if self.arrows != Arrows::Static {
                        continue;
                    }

                    let left_paragraph = Paragraph::new("▶")
                        .alignment(Alignment::Right)
                        .style(Style::default().fg(HIGHLIGHT_COLOR));

                    let right_paragraph = Paragraph::new("◀")
                        .alignment(Alignment::Left)
                        .style(Style::default().fg(HIGHLIGHT_COLOR));

                    frame.render_widget(left_paragraph, left_side);
                    frame.render_widget(right_paragraph, right_side);
                } else {
                    let option_paragraph = Paragraph::new(option.to_string())
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(PRIMARY_COLOR));
                    frame.render_widget(option_paragraph, option_area);
                }
            }
        }
    }
}

// #[derive(Clone)]
// pub struct SearchPopup {
//     pub toggled: bool,
//     pub query: String,
// }
