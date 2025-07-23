use std::sync::{Arc, Mutex};

use crate::{
    app::Action,
    config::{ERROR_COLOR, HIGHLIGHT_COLOR, PRIMARY_COLOR, SECONDARY_COLOR},
    mal::{models::anime::Anime, MalClient},
    screens::ExtraInfo,
    utils::{imageManager::ImageManager, terminalCapabilities::TERMINAL_RATIO},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::{self, border},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
};
use std::cmp::min;
use tui_widgets::big_text::{BigText, PixelSize};

use super::navigatable::Navigatable;

const AVAILABLE_SEASONS: [&str; 4] = ["Winter", "Spring", "Summer", "Fall"];
const FIRST_YEAR: u16 = 1917;
const FIRST_SEASON: &str = "Winter";
const BUTTON_HEIGHT: u16 = 3;
const RATIO: f32 = 422.0 / 598.0;

#[derive(PartialEq, Clone, Debug)]
enum Focus {
    PlayButtons,
    StatusButtons,
    Synopsis,
}

#[derive(Clone)]
pub struct AnimePopup {
    anime: Anime,
    toggled: bool,
    buttons: Vec<&'static str>,
    button_nav: Navigatable,
    status_buttons: Vec<SelectionPopup>,
    status_nav: Navigatable,
    image_manager: Arc<Mutex<ImageManager>>,
    focus: Focus,
}

impl AnimePopup {
    pub fn new(info: ExtraInfo) -> Self {
        let buttons = vec!["Play", "Share"];
        let image_manager = Arc::new(Mutex::new(ImageManager::new()));
        ImageManager::init_with_threads(&image_manager, info.app_sx.clone());

        Self {
            image_manager,
            anime: Anime::empty(),
            toggled: false,
            button_nav: Navigatable::new((buttons.len() as u16, 1)),
            status_nav: Navigatable::new((1, 3)),
            status_buttons: Vec::new(),
            buttons,
            focus: Focus::PlayButtons,
        }
    }

    pub fn set_anime(&mut self, anime: Anime) -> &Self {
        self.anime = anime;

        let episode_options: Vec<String> = (0..=self.anime.num_episodes.max(1))
            .map(|i| i.to_string())
            .collect();

        self.status_buttons = vec![
            SelectionPopup::new()
                .add_option("Add to list")
                .add_option("Watching")
                .add_option("Plan to watch")
                .add_option("Completed")
                .add_option("On-Hold")
                .add_option("Dropped")
                .with_arrows(Arrows::Static)
                .with_selected_option(self.anime.my_list_status.status.to_string())
                .clone(),
            SelectionPopup::new()
                .add_option("Rate:")
                .add_options(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"])
                .with_arrows(Arrows::Static)
                .with_selected_option(
                    self.anime
                        .my_list_status
                        .score
                        .map_or("Not Rated".to_string(), |s| s.to_string()),
                )
                .clone(),
            SelectionPopup::new()
                .add_options(episode_options)
                .with_arrows(Arrows::Static)
                .with_selected_option(self.anime.my_list_status.num_episodes_watched.to_string())
                .with_displaying_format(format!("{{}} / {}", self.anime.num_episodes))
                .clone(),
        ];
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
        match self.focus {
            Focus::PlayButtons => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.focus = Focus::StatusButtons;
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.focus = Focus::Synopsis;
                        }
                        _ => {}
                    }
                    return None;
                }

                match key_event.code {
                    KeyCode::Char('k') | KeyCode::Down => {
                        self.button_nav.move_down();
                    }
                    KeyCode::Char('j') | KeyCode::Up => {
                        self.button_nav.move_up();
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        self.button_nav.move_right();
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        self.button_nav.move_left();
                    }
                    KeyCode::Char('q') => {
                        self.close();
                    }
                    KeyCode::Enter => {
                        if let Some(button) = self.button_nav.get_selected_item(&self.buttons) {
                            match *button {
                                "Play" => {
                                    return Some(Action::PlayAnime(self.anime.clone()));
                                }
                                "Share" => {
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Focus::StatusButtons => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('l')
                        | KeyCode::Right
                        | KeyCode::Char('k')
                        | KeyCode::Down => {
                            self.focus = Focus::PlayButtons;
                            if let Some(button) = self
                                .status_nav
                                .get_selected_item_mut(&mut self.status_buttons)
                            {
                                button.close();
                            }
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.focus = Focus::Synopsis;
                            if let Some(button) = self
                                .status_nav
                                .get_selected_item_mut(&mut self.status_buttons)
                            {
                                button.close();
                            }
                        }
                        _ => {}
                    }
                    return None;
                }

                if let Some(dropdown) = self
                    .status_nav
                    .get_selected_item_mut(&mut self.status_buttons)
                {
                    match (dropdown.is_open(), key_event.code) {
                        (false, KeyCode::Char('l') | KeyCode::Right) => {
                            self.status_nav.move_right();
                            return None;
                        }
                        (false, KeyCode::Char('h') | KeyCode::Left) => {
                            self.status_nav.move_left();
                            return None;
                        }
                        (false, KeyCode::Char('q')) => {
                            self.close();
                            return None;
                        }
                        _ => if let Some(selection) = dropdown.handle_input(key_event) {},
                    }
                }
            }

            Focus::Synopsis => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.focus = Focus::PlayButtons;
                        }
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.focus = Focus::StatusButtons;
                        }
                        KeyCode::Char('q') => {
                            self.close();
                        }
                        _ => {}
                    }
                    return None;
                }
            }
        }

        None
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
                    .style(Style::default().fg(
                        if highlighted && self.focus == Focus::PlayButtons {
                            HIGHLIGHT_COLOR
                        } else {
                            SECONDARY_COLOR
                        },
                    ));
                frame.render_widget(button_paragraph, area);
            });

        // the rest of the popup
        // the image
        let image_height = bottom_area.y - popup_area.y - 3;
        let image_width = (image_height as f32 * RATIO * TERMINAL_RATIO) as u16;
        let image_area = Rect {
            x: popup_area.x + 4,
            y: popup_area.y + 2,
            width: image_width,
            height: image_height,
        };

        ImageManager::render_image(&self.image_manager, &self.anime, frame, image_area, true);

        //title and info area
        let [title_area, info_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .areas(popup_area.inner(Margin::new(1, 1)));
        let title_area_x = image_area.x + image_area.width + 3;
        let title_area = Rect {
            x: title_area_x,
            y: title_area.y,
            width: popup_area.x + popup_area.width - title_area_x - 2,
            height: title_area.height,
        };
        let info_area = Rect {
            x: title_area.x,
            y: info_area.y,
            width: title_area.width,
            height: info_area.height - buttons_area.height,
        };

        let title = if self.anime.alternative_titles.en.is_empty() {
            self.anime.title.clone()
        } else {
            self.anime.alternative_titles.en.clone()
        };
        let title_text = Paragraph::new(title)
            .alignment(Alignment::Center)
            .style(Style::default().fg(SECONDARY_COLOR).bold());

        frame.render_widget(title_text, title_area.inner(Margin::new(0, 1)));

        //synopsis
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
                    .style(Style::default().fg(if self.focus == Focus::Synopsis {
                        HIGHLIGHT_COLOR
                    } else {
                        PRIMARY_COLOR
                    })),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(PRIMARY_COLOR));
        frame.render_widget(synopsis_text, synopsis_area);

        // score text
        let big_text = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .lines(vec![self.anime.mean.to_string().into()])
            .build();
        let [_, big_text_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(20)])
            .areas(info_area);
        frame.render_widget(big_text, big_text_area);

        // info area
        let info_texts = ["Ranked", "Popularity", "Members"];
        let info_values = [
            self.anime.rank.to_string(),
            self.anime.popularity.to_string(),
            self.anime.num_list_users.to_string(),
        ];

        let info_texts_2 = vec![
            self.anime.start_season.to_string(),
            self.anime.media_type.to_string(),
            self.anime.studios_as_string(),
        ];

        let mut total_length = info_area.x + (info_area.width / 6);
        let y_pos = info_area.y + info_area.height * 1/5; 
        const SPACE: u16 = 8; 
        for (i, (text, value)) in info_texts.iter().zip(info_values.iter()).enumerate() {
            let info_text_area = Rect {
                x: total_length,
                y: y_pos,
                width: text.len() as u16,
                height: 1,
            };
            total_length += text.len() as u16 + 1; // +1 for margin 
            let text_paragraph = Paragraph::new(*text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White));
            frame.render_widget(text_paragraph, info_text_area);

            let info_text_area = Rect {
                x: total_length,
                y: y_pos,
                width: value.len() as u16 + 1,
                height: 1,
            };
            total_length += value.len() as u16 + SPACE;
            let value_text = Paragraph::new(if i == info_texts.len() - 1 {
                value.clone()
            } else {
                format!("#{}", value)
            })
            .alignment(Alignment::Center)
            .style(Style::default().fg(SECONDARY_COLOR));
            frame.render_widget(value_text, info_text_area);
        }

        let uppeline_width = total_length;

        total_length = info_area.x + (info_area.width / 6);
        for (i, text) in info_texts_2.iter().enumerate() {
            let info_text_area = Rect {
                x: total_length,
                y: y_pos + 3,
                width: text.len() as u16,
                height: 1,
            };
            total_length += text.len() as u16;
            let text_paragraph = Paragraph::new(text.clone())
                .alignment(Alignment::Center)
                .style(Style::default().fg(SECONDARY_COLOR));
            frame.render_widget(text_paragraph, info_text_area);

            if i == info_texts_2.len() - 1 {
                continue;
            }
            let separator_area = Rect {
                x: total_length + SPACE/2,
                y: y_pos + 3,
                width: 1,
                height: 1,
            };
            total_length += SPACE + 1;
            let separator_paragraph = Paragraph::new("|")
                .alignment(Alignment::Center)
                .style(Style::default().fg(SECONDARY_COLOR));
            frame.render_widget(separator_paragraph, separator_area);
        }

        // let block = Block::default()
        //     .borders(Borders::ALL)
        //     .border_set(border::ROUNDED)
        //     .title("Anime Info")
        //     .style(Style::default().fg(PRIMARY_COLOR));
        // frame.render_widget(block, info_area);

        // buttons within info area
        let status_buttons_area = Rect {
            x: info_area.x + (info_area.width / 10),
            y: info_area.y + info_area.height - 4,
            width: info_area.width * 8 / 10,
            height: 3,
        };

        self.status_nav.construct(
            &self.status_buttons,
            status_buttons_area,
            |dropdown, area, highlighted| {
                dropdown.render(
                    frame,
                    area,
                    highlighted && self.focus == Focus::StatusButtons,
                );
            },
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
    displaying_format: String,
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
            displaying_format: String::new(),
        }
    }

    pub fn with_arrows(mut self, arrow_type: Arrows) -> Self {
        self.arrows = arrow_type;
        self
    }

    pub fn with_selected_option(mut self, option: String) -> Self {
        if let Some(index) = self
            .options
            .iter()
            .position(|o| o.to_lowercase() == option.to_lowercase())
        {
            self.selected_index = index;
            self.next_index = index;
        } else {
            self.selected_index = 0;
            self.next_index = 0;
        }
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

    pub fn add_options<I, S>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for option in options {
            self = self.add_option(option);
        }
        self
    }

    pub fn with_displaying_format<T: Into<String>>(mut self, text: T) -> Self {
        self.displaying_format = text.into();
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
        let option = self
            .options
            .get(self.selected_index)
            .unwrap_or(&"No options available".to_string())
            .clone();
        let option = if self.displaying_format.is_empty() {
            option
        } else {
            self.displaying_format.replace("{}", &option)
        };

        let filter = Paragraph::new(option)
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

#[derive(Clone)]
pub struct ErrorPopup {
    toggled: bool,
    error_message: String,
    height: u16,
    width: u16,
}
impl ErrorPopup {
    pub fn new() -> Self {
        Self {
            toggled: false,
            error_message: String::new(),
            height: 10,
            width: 40,
        }
    }

    pub fn toggle(mut self, message: String) -> Self {
        self.toggled = !self.toggled;
        self.error_message = message;
        self
    }

    pub fn is_open(&self) -> bool {
        self.toggled
    }

    pub fn set_error(&mut self, message: String) -> &Self {
        let content_width = self.width - 2;
        let total_lines: u16 = message
            .lines()
            .map(|line| {
                if line.is_empty() {
                    1
                } else {
                    (line.len() as u16).div_ceil(content_width)
                }
            })
            .sum();

        self.height = total_lines + 2;
        self.error_message = message;
        self
    }

    pub fn open(&mut self) -> &Self {
        self.toggled = true;
        self
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        if !self.toggled {
            return None;
        }
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.toggled = false;
                self.error_message.clear();
                None
            }
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.toggled {
            return;
        }

        let area = frame.area();
        let popup_area = Rect::new(
            area.x + (area.width - self.width) / 2,
            area.y + (area.height - self.height) / 2,
            self.width,
            self.height,
        );
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .title("Error")
            .style(Style::default().fg(ERROR_COLOR));
        frame.render_widget(block.clone(), popup_area);

        let paragraph = Paragraph::new(self.error_message.clone())
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(ERROR_COLOR));
        frame.render_widget(paragraph, popup_area);
    }
}

// #[derive(Clone)]
// pub struct SearchPopup {
//     pub toggled: bool,
//     pub query: String,
// }
