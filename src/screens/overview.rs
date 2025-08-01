use std::cell::RefCell;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

use super::widgets::navbar::NavBar;
use super::{screens::*, BackgroundUpdate, ExtraInfo, Screen};
use crate::config::PRIMARY_COLOR;
use crate::mal::models::anime::Anime;
use crate::app::{Action, Event};
use crate::utils::terminalCapabilities::get_picker;
use ratatui::layout::{Margin, Rect};
use ratatui::widgets::{Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::{
    layout::{Constraint, Direction, Layout}, style::{Color, Style}, widgets::{Block,  Borders, Clear}, Frame,
    symbols,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui_image::{
    thread::{ResizeRequest, ResizeResponse, ThreadProtocol},
    StatefulImage,
};


#[derive(Clone)]
pub struct OverviewScreen { 
    loading: bool,
    animes: Vec<Anime>,
    scroll_offset: u16,
    app_info: ExtraInfo,
}

impl OverviewScreen {
    pub fn new(info: ExtraInfo) -> Self {
        Self {
            animes: vec![
                Anime::empty(),
                Anime::empty(),
                Anime::empty(),
                Anime::empty(),
            ],

            scroll_offset: 0,
            app_info: info,
            loading: false,
            // async_state: ThreadProtocol::new(sx_worker, Some(picker.new_resize_protocol(dyn_img))),
        }
    }
}

impl Screen for OverviewScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);


        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Percentage(100),
                ]
            )
            .areas(area);



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
        if self.loading {
            return None;
        }
        self.loading = true; 
        let info = self.app_info.clone();

        // let rx = Arc::clone(&self.rx);
        let id = self.get_name();

        Some(thread::spawn(move || {
            let update = BackgroundUpdate::new(id)
            .set("anime", Anime::empty());

            thread::sleep(std::time::Duration::from_secs(2));
            // let _ = info.mal_client.test();
            let _ = info.app_sx.send(Event::BackgroundNotice(update));
        }))
    }

    fn apply_update(&mut self, update: BackgroundUpdate) {
        // Handle updates from the background thread if needed
        // For now, we do nothing
    }
}
