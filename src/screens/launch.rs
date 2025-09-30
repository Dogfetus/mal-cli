use super::{
    ExtraInfo, Screen,
    screens::*,
    widgets::{button::Button, navigatable::Navigatable},
};
use crate::app::Action;
use crate::mal::MalClient;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Clear, Paragraph},
};

pub struct LaunchScreen {
    buttons: Vec<&'static str>,
    navigatable: Navigatable,
}

impl LaunchScreen {
    pub fn new(_: ExtraInfo) -> Self {
        Self {
            buttons: vec![
                "Browse",
                if !MalClient::user_is_logged_in() {
                    "Log In"
                } else {
                    "Log Out"
                },
                "Exit",
            ],
            navigatable: Navigatable::new((3, 1)),
        }
    }

    fn activate_button(&self, index: usize) -> Option<Action> {
        match index {
            0 => {
                if MalClient::user_is_logged_in() {
                    Some(Action::SwitchScreen(OVERVIEW))
                } else {
                    Some(Action::ShowError("Please log in to browse".to_string()))
                }
            }
            1 => {
                if MalClient::user_is_logged_in() {
                    MalClient::log_out();
                    Some(Action::SwitchScreen(LAUNCH))
                } else {
                    Some(Action::SwitchScreen(LOGIN))
                }
            }
            2 => Some(Action::Quit),
            _ => None,
        }
    }
}

impl Screen for LaunchScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        frame.render_widget(Clear, area);

        let page_chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let button_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(20),
                Constraint::Fill(1),
            ])
            .split(page_chunk[1]);

        let button_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Length((self.buttons.len() * 3) as u16),
                Constraint::Percentage(80),
            ])
            .split(button_area[1]);

        let centeded_chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Percentage(30)])
            .split(page_chunk[0]);

        let header_text = [
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

        self.navigatable
            .construct(&self.buttons, button_area[1], |button, area, iselected| {
                Button::new(button).selected(iselected).render(frame, area);
            });
    }

    fn handle_keyboard(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('j') => {
                self.navigatable.move_up();
            }
            KeyCode::Down | KeyCode::Char('k') => {
                self.navigatable.move_down();
            }
            KeyCode::Enter => {
                return self.activate_button(self.navigatable.get_selected_index());
            }
            _ => {}
        };

        None
    }

    fn handle_mouse(&mut self, mouse_event: MouseEvent) -> Option<Action> {
        if let Some(index) = self.navigatable.get_hovered_index(mouse_event) {
            if let crossterm::event::MouseEventKind::Down(_) = mouse_event.kind {
                return self.activate_button(index);
            }
        };

        None
    }

    fn uses_navbar(&self) -> bool {
        false
    }
}
