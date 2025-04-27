use super::Screen;
use crate::ui::widgets::button::Button;
use crate::app::{App, Action};
use ratatui::{
    style::{Style,Color},
    widgets::{Paragraph, Clear},
    layout::{Constraint, Direction, Layout, Alignment},
    Frame, 
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::mal; 


pub struct LoginPage { 
    selected_button: usize,
    buttons: Vec<&'static str>,
}

impl LoginPage {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            buttons: vec![
                "Copy",
                "Paste",
                "Back",
            ],
        }
    }
}

impl Screen for LoginPage {
    #[allow(unused)]
    fn draw(&self, frame: &mut Frame, app: &App) {
        let area = frame.area();

        frame.render_widget(Clear, area);

        let page_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(65),
            Constraint::Percentage(35),
        ])
        .split(area);

        let centeded_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Percentage(30),
        ])
        .split(page_chunk[0]);


        let header_text = vec![
        ];

        let alpha = Paragraph::new(header_text.join("\n"))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

        frame.render_widget(alpha, area);

        for (i, button) in self.buttons.iter().enumerate() {
            Button::new(button)
                .offset((0, -1 + (i as i16 * 3)))
                .center_x()
                .selected(i == self.selected_button)
                .render(frame, page_chunk[1]);
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
                    _ => { return Some(Action::SwitchScreen("Launch")); }
                }
            }
            _ => {} 
        };

        None
    }
}
