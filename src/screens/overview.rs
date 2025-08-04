use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};

use super::widgets::navbar::NavBar;
use super::widgets::navigatable::{self, Navigatable};
use super::{BackgroundUpdate, ExtraInfo, Screen, screens::*};
use crate::app::{Action, Event};
use crate::config::PRIMARY_COLOR;
use crate::mal::models::anime::Anime;
use crate::utils::terminalCapabilities::get_picker;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Margin, Rect};
use ratatui::widgets::{Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Clear},
};
use ratatui_image::{
    StatefulImage,
    thread::{ResizeRequest, ResizeResponse, ThreadProtocol},
};

#[derive(Clone)]
pub struct OverviewScreen {
    bg_loaded: bool,
    animes: Vec<Anime>,
    scroll_offset: u16,
    app_info: ExtraInfo,
    navigation: Navigatable,
}

impl OverviewScreen {
    pub fn new(info: ExtraInfo) -> Self {
        Self {
            animes: vec![
                Anime::example(1),
                Anime::example(2),
                Anime::example(3),
                Anime::example(4),
                Anime::example(5),
            ],

            scroll_offset: 0,
            app_info: info,
            bg_loaded: false,
            navigation: Navigatable::new((1, 3)),
        }
    }
}

impl Screen for OverviewScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let [_, content] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Fill(1)])
            .areas(area);

        self.navigation
            .construct(&self.animes, content, |anime, area, highlighted| {
                let b = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(PRIMARY_COLOR))
                    .title(anime.title.clone())
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .style(if highlighted {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    });

                frame.render_widget(b, area);
            });
    }

    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('j') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('k') => {
                self.scroll_offset += 1;
            }
            KeyCode::Enter => {
                return Some(Action::NavbarSelect(true));
            }
            _ => {}
        };
        None
    }

    fn background(&mut self) -> Option<JoinHandle<()>> {
        if self.bg_loaded {
            return None;
        }
        self.bg_loaded = true;
        let info = self.app_info.clone();
        let id = self.get_name();

        let app_dir = std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".local/share/mal-cli"))
            .expect("Failed to get app directory");
        let log_file = app_dir.join("watch_history");

        Some(thread::spawn(move || {
            if let Ok(file) = OpenOptions::new().open(log_file) {

                // read line by line
                let content = BufReader::new(file);
                for line in content.lines() {
                    if let Ok(line) = line {}
                    // TODO: idk if this will be necessary, considering this may or may not reuslt
                    // in me fetching multiple requests a second, unless only the title is
                    // necesarry which might be good enough. we'll see
                }
            }
        }))
    }

    fn apply_update(&mut self, update: BackgroundUpdate) {
        // Handle updates from the background thread if needed
        // For now, we do nothing
    }
}
