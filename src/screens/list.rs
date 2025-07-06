use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

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
use ratatui::symbols;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;

use super::screens::*;
use super::widgets::animebox::LongAnimeBox;
use super::widgets::navbar::NavBar;
use super::widgets::navigatable::Navigatable;
use super::widgets::popup::Arrows;
use super::widgets::popup::SelectionPopup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    NavBar,
    Content,
    Search,
    Filter,
    List,
}

#[derive(Clone)]
pub struct ListScreen {
    animes: Vec<Anime>,
    list_type: String,

    bg_loaded: bool,
    image_manager: Arc<Mutex<ImageManager>>,

    focus: Focus,
    navbar: NavBar,

    filter_popup: SelectionPopup,
    list_popup: SelectionPopup,
    search_input: Input,
    navigatable: Navigatable,
}

impl ListScreen {
    pub fn new() -> Self {
        Self {
            image_manager: Arc::new(Mutex::new(ImageManager::new())),
            navigatable: Navigatable::new((3, 3)),
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
            list_popup: SelectionPopup::new()
                .with_arrows(Arrows::Static)
                .add_option("all")
                .add_option("currently watching")
                .add_option("plan to watch")
                .add_option("completed")
                .add_option("on hold")
                .add_option("dropped"),
            search_input: Input::new(),
            navbar: NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(SEARCH)
                .add_screen(LIST)
                .add_screen(PROFILE),
            animes: Vec::new(),
            focus: Focus::Content,
            bg_loaded: false,
            list_type: "all".to_string(),
        }
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

        let [options, _, content] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(bottom);

        let [search, list, filter] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .areas(options);

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
            area.x,
            content.y,
            side.width * 8 / 10,
            content.height * 3 / 10,
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

        let info_value = Paragraph::new("0\n0\n")
            .alignment(Alignment::Left)
            .style(style::Style::default().fg(Color::DarkGray));
        frame.render_widget(info, info_area_left.inner(Margin::new(1, 0)));
        frame.render_widget(info_value, info_area_right.inner(Margin::new(1, 1)));

        self.navigatable
            .construct(&self.animes, content, |anime, area, highlight| {
                LongAnimeBox::render(
                    anime,
                    &self.image_manager,
                    frame,
                    area,
                    highlight && self.focus == Focus::Content,
                );
            });

        self.filter_popup
            .render(frame, filter, self.focus == Focus::Filter);
        self.list_popup
            .render(frame, list, self.focus == Focus::List);
        self.navbar.render(frame, top);
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
                            self.focus = Focus::List;
                            return None;
                        }
                        _ => {}
                    }
                }

                if let Some(text) = self.search_input.handle_event(key_event) {
                    if !text.is_empty() {}
                }
            }

            Focus::List => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('k') | KeyCode::Down => {
                            self.focus = Focus::Content;
                            self.list_popup.close();
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.focus = Focus::Search;
                            self.list_popup.close();
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.focus = Focus::Filter;
                            self.list_popup.close();
                        }
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.focus = Focus::NavBar;
                            self.navbar.select();
                            self.list_popup.close();
                        }
                        _ => {}
                    }
                } else {
                    if let Some(mut list_type) = self.list_popup.handle_input(key_event) {
                        self.list_type = list_type.clone();
                        if list_type == "currently watching" {
                            list_type = "watching".to_string();
                        }
                    }
                }
            }

            Focus::Filter => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('k') | KeyCode::Down => {
                            self.focus = Focus::Content;
                            self.filter_popup.close();
                        }
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.focus = Focus::NavBar;
                            self.navbar.select();
                            self.filter_popup.close();
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.focus = Focus::List;
                            self.filter_popup.close();
                        }
                        _ => {}
                    }
                } else {
                    if let Some(mut filter_type) = self.filter_popup.handle_input(key_event) {
                        if filter_type == "popularity" {
                            filter_type = "bypopularity".to_string();
                        }
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
                        _ => {}
                    }
                } else {
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

    fn background(&mut self, info: super::BackgroundInfo) -> Option<JoinHandle<()>> {
        if self.bg_loaded {
            return None;
        }
        self.bg_loaded = true;

        let id = self.get_name();
        ImageManager::init_with_dedicated_thread(
            &self.image_manager,
            info.app_sx.clone(),
            id.clone(),
        );
        // let image_manager = self.image_manager.clone();

        Some(std::thread::spawn(move || {
            if let Some(animes) = info.mal_client.get_anime_list(None, 0, 1000) {
                let update = super::BackgroundUpdate::new(id).set("animes", animes);
                info.app_sx
                    .send(super::Event::BackgroundNotice(update))
                    .unwrap();
            }
        }))
    }

    fn apply_update(&mut self, mut update: super::BackgroundUpdate) {
        if let Some(animes) = update.take::<Vec<Anime>>("animes") {
            self.animes = animes;
        }
    }
}
