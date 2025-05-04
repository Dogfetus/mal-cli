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
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        frame.render_widget(Clear, area);

        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(4),
                    Constraint::Length(2),
                    Constraint::Percentage(100),
                ]
                .as_ref(),
            )
            .split(area);

        let header_text = vec![
            "█▄▀▄█ ▄▀▀▄ █        ▄▀▀▀ █    ▀█▀",
            "█ ▀ █ █▀▀█ █    ▀▀  █    █     █ ",
            "▀   ▀ ▀  ▀ ▀▀▀▀      ▀▀▀ ▀▀▀▀ ▀▀▀"
        ];
        let header_area = Rect::new(
            area[0].x,
            area[0].y,
            area[0].width-2,
            area[0].height
        );

        let header = Paragraph::new(header_text.join("\n"))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        frame.render_widget(header, header_area);


        // Calculate button width and spacing
        let button_spacing = 2; // Space between buttons
        let button_area = area[1];
        let button_width = 12;
        let button_height = 1; // Single line height
        let total_width = (button_width * self.buttons.len() as u16) + 
                        (button_spacing * (self.buttons.len() - 1) as u16);
        let start_x = (button_area.width - total_width) / 2;

        for (i, button) in self.buttons.iter().enumerate() {
            let x_pos = button_area.x + start_x + (i as u16 * (button_width + button_spacing));
            let button_rect = Rect::new(x_pos, button_area.y, button_width, button_height);

            // Add Unicode symbols for better-looking compact buttons
            let display_text = if i == self.selected_button {
                format!("{}", button)
            } else {
                button.to_string()
            };

            let style = if i == self.selected_button {
                Style::default().fg(Color::Cyan).bg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let p = Paragraph::new(display_text)
                .alignment(Alignment::Center)
                .style(style);

            frame.render_widget(p, button_rect);
        }

        // Define the size and position for the anime box
        // You can adjust these values to place the box wherever you want
        let box_width = 40; // Smaller width
        let box_height = 18; // Total height
        let x_pos = 10; // X position from left
        let y_pos = 5;  // Y position from top

        // Create a rect for the anime box
        let anime_box_rect = Rect::new(x_pos, y_pos, box_width, box_height);

        // Split the anime box into sections
        let anime_box_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Title bar
                Constraint::Length(13), // Content area
                Constraint::Length(3),  // Bottom info
            ])
            .split(anime_box_rect);

        // Split the content area horizontally
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Image area
                Constraint::Percentage(60), // Info area
            ])
            .split(anime_box_layout[1]);

        // Title section
        let title = "My Anime Title";
        let title_bar = Paragraph::new(title)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Cyan)));

        // Image placeholder
        let image_placeholder = Paragraph::new("IMG")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(Color::Cyan)));

        // Info section - condensed to fit smaller space
        let info = vec![
            "Score: 7.55",
            "Rank: #1783",
            "Pop: #717",
            "",
            "Synopsis:",
            "Short anime",
            "description"
        ];

        let info_section = Paragraph::new(info.join("\n"))
            .style(Style::default().fg(Color::White))
            .block(Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(Color::Cyan)));

        // Bottom info section - condensed
        let bottom_section = Paragraph::new("Eps: 0/12 • Fall 2021")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Cyan)));

        // Render all components
        frame.render_widget(title_bar, anime_box_layout[0]);
        frame.render_widget(image_placeholder, content_layout[0]);
        frame.render_widget(info_section, content_layout[1]);
        frame.render_widget(bottom_section, anime_box_layout[2]);
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
