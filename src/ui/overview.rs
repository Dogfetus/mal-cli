use super::{screens::*, Screen};
use crate::ui::widgets::button::Button;
use crate::app::Action;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span, Text}, widgets::{ Block, Borders, Clear, Paragraph, Wrap}, Frame 
};
use crossterm::event::{KeyCode, KeyEvent};
use std::cmp::{max, min};


#[derive(Clone)]
pub struct OverviewScreen { 
    animes: Vec<&'static str>,
    buttons: Vec<&'static str>,
    selected_button: usize,
}

impl OverviewScreen {
    pub fn new() -> Self {
        Self {
            animes: vec![
                "one piece",
                "naruto",
                "bleach",
                "attack on titan",
                "hunter x hunter",
            ],

            buttons: vec![
                "User",
                "Settings",
                "Info",
                "Back",
            ],

            selected_button: 0,
        }
    }
}

impl Screen for OverviewScreen {
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        frame.render_widget(Clear, area);

        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20), 
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(area);

        for (i, anime) in self.animes.iter().enumerate() {
            let block = Block::default()
                .title(format!("{}: {}", i, anime))
                .borders(Borders::ALL);
            frame.render_widget(block, area[i]);
        }

        for (i, button) in self.buttons.iter().enumerate() {
                Button::new(button)
                .center()
                .selected(i == self.selected_button)
                .offset((0, 3*i as i16))
                .render(frame, frame.area());
        }

    }

    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('j') => {
                if self.selected_button > 0 {
                    self.selected_button -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('k') => {
                if self.selected_button < self.buttons.len() - 1 {
                    self.selected_button += 1;
                }
            }
            KeyCode::Enter => {
                match self.selected_button {
                    0 => return Some(Action::SwitchScreen(PROFILE)),
                    1 => return Some(Action::SwitchScreen(SETTINGS)),
                    2 => return Some(Action::SwitchScreen(INFO)),
                    3 => return Some(Action::SwitchScreen(LAUNCH)),
                    _ => {}
                }
            }
            _ => {} 
        };
        None
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
    }
}

