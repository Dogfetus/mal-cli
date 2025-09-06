use super::{screens::*, ExtraInfo, Screen};
use crate::screens::widgets::button::Button; 
use crate::app::Action;
use crate::mal::MalClient;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Style}, widgets::{Clear, Paragraph}, Frame 
};
use crossterm::event::{KeyCode, KeyEvent};


pub struct LaunchScreen { 
    selected_button: usize,
    buttons: Vec<&'static str>,
}

impl LaunchScreen {
    pub fn new(_: ExtraInfo) -> Self {
        Self {
            selected_button: 0,
            buttons: vec![
                "Browse",
                if !MalClient::user_is_logged_in() { "Log In" } else { "Log Out" },
                "Exit",
            ],
        }
    }
}

impl Screen for LaunchScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        frame.render_widget(Clear, area);

        let page_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
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
            " ███╗   ███╗ █████╗ ██╗                ██████╗██╗     ██╗ ",
            " ████╗ ████║██╔══██╗██║               ██╔════╝██║     ██║ ",
            " ██╔████╔██║███████║██║     ███████╗  ██║     ██║     ██║ ",
            " ██║╚██╔╝██║██╔══██║██║     ╚══════╝  ██║     ██║     ██║ ",
            " ██║ ╚═╝ ██║██║  ██║███████╗          ╚██████╗███████╗██║ ",
            " ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝           ╚═════╝╚══════╝╚═╝ ",
        ];

        let alpha = Paragraph::new(header_text.join("\n"))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

        frame.render_widget(alpha, centeded_chunk[1]);

        for (i, button) in self.buttons.iter().enumerate() {
            Button::new(button)
                .offset((0, -1 + (-3)*(self.buttons.len() as i16)/2 + (i as i16 * 3)))
                .center()
                .selected(i == self.selected_button)
                .render(frame, page_chunk[1]);
        }
    }

    fn handle_keyboard(&mut self, key_event: KeyEvent) -> Option<Action> {
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
                    0 => 
                    if MalClient::user_is_logged_in() {
                        return Some(Action::SwitchScreen(OVERVIEW));
                    } else {
                        return Some(Action::ShowError("Please log in to browse".to_string()));
                    },
                    1 => {

                        // return None;
                        if MalClient::user_is_logged_in() {
                            MalClient::log_out();
                            return Some(Action::SwitchScreen(LAUNCH));
                        }

                        return Some(Action::SwitchScreen(LOGIN))
                    },

                    2 => return Some(Action::Quit),
                    _ => {}
                }
            }
            _ => {} 
        };

        None
    }

    fn uses_navbar(&self) -> bool { false }
}
