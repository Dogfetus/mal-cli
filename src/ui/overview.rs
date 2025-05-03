use super::{screens::*, Screen};
use crate::ui::widgets::button::Button;
use crate::app::Action;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span, Text}, widgets::{ Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap}, Frame 
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
//     fn draw(&self, frame: &mut Frame) {
//     let area = frame.area();
//     frame.render_widget(Clear, area);
//
//     // Create layout with more space on top
//     let main_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Length(2),    // Title bar
//             Constraint::Length(2),    // Nav bar
//             Constraint::Length(2),    // Spacing
//             Constraint::Min(10),      // Content area
//             Constraint::Length(1),    // Footer
//         ])
//         .split(area);
//
//     // Title bar
//     let title = Paragraph::new("Anime Tracker")
//         .style(Style::default().fg(Color::White).bg(Color::Blue))
//         .alignment(Alignment::Center);
//     frame.render_widget(title, main_layout[0]);
//
//     // Nav bar
//     let nav = Paragraph::new("Home | Browse | My List | Settings")
//         .style(Style::default().bg(Color::DarkGray))
//         .alignment(Alignment::Center);
//     frame.render_widget(nav, main_layout[1]);
//
//     // Content area with 5 cards
//     let content_chunks = Layout::default()
//         .direction(Direction::Horizontal)
//         .margin(1)
//         .constraints([
//             Constraint::Percentage(18),
//             Constraint::Percentage(2),
//             Constraint::Percentage(18),
//             Constraint::Percentage(2),
//             Constraint::Percentage(18),
//             Constraint::Percentage(2),
//             Constraint::Percentage(18),
//             Constraint::Percentage(2),
//             Constraint::Percentage(18),
//         ])
//         .split(main_layout[3]); // Using index 3 to add space
//
//     // Create 5 anime cards without heavy borders
//     for (i, anime) in self.animes.iter().enumerate().take(5) {
//         let card_idx = i * 2; // Skip spacing chunks
//
//         let card_layout = Layout::default()
//             .direction(Direction::Vertical)
//             .constraints([
//                 Constraint::Length(2),  // Title
//                 Constraint::Length(8),  // Image
//                 Constraint::Length(5),  // Info
//                 Constraint::Length(2),  // Button
//             ])
//             .split(content_chunks[card_idx]);
//
//         // Card background - highlight if selected
//         let is_selected = i == self.selected_button;
//         let bg_style = if is_selected {
//             Style::default().bg(Color::DarkGray)
//         } else {
//             Style::default().bg(Color::Black)
//         };
//
//         let card_bg = Block::default()
//             .style(bg_style)
//             .padding(Padding::new(1, 1, 0, 0));
//         frame.render_widget(card_bg, content_chunks[card_idx]);
//
//         // Card title - no borders
//         let title_style = if is_selected {
//             Style::default().fg(Color::Yellow)
//         } else {
//             Style::default()
//         };
//
//         let anime_title = Paragraph::new(anime.to_string())
//             .alignment(Alignment::Center)
//             .style(title_style);
//         frame.render_widget(anime_title, card_layout[0]);
//
//         // Card image - simple background, no borders
//         let image = Block::default()
//             .style(Style::default().bg(Color::Black));
//         frame.render_widget(image, card_layout[1]);
//
//         // Card info - no borders
//         let info = Paragraph::new(format!("Studio: {}\nSource: {}", 
//             anime, anime));
//         frame.render_widget(info, card_layout[2]);
//
//         // Button - highlighted when selected
//         let button_style = if is_selected {
//             Style::default().fg(Color::Yellow).bg(Color::Blue)
//         } else {
//             Style::default().bg(Color::DarkGray)
//         };
//
//         frame.render_widget(
//             Paragraph::new("Add to List")
//                 .alignment(Alignment::Center)
//                 .style(button_style),
//             card_layout[3]
//         );
//     }
//
//     // Footer
//     let footer = Paragraph::new("Press ← → to select, Enter to view details")
//         .style(Style::default().fg(Color::White))
//         .alignment(Alignment::Center);
//     frame.render_widget(footer, main_layout[4]);
// }
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


// fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
//     let popup_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Percentage((100 - percent_y) / 2),
//             Constraint::Percentage(percent_y),
//             Constraint::Percentage((100 - percent_y) / 2),
//         ])
//         .split(r);
//
//     Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([
//             Constraint::Percentage((100 - percent_x) / 2),
//             Constraint::Percentage(percent_x),
//             Constraint::Percentage((100 - percent_x) / 2),
//         ])
//         .split(popup_layout[1])[1]
// }
