use crate::{app::Action, screens::Screen};
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;

use super::screens::*;
use super::widgets::navbar::NavBar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    NavBar,
    Content,
}

#[derive(Clone)]
pub struct ListScreen {
    navbar: NavBar,
    focus: Focus,
}

impl ListScreen {
    pub fn new() -> Self {
        Self {
            focus: Focus::Content,
            navbar: NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(SEARCH)
                .add_screen(LIST)
                .add_screen(PROFILE),
        }
    }
}

impl Screen for ListScreen {
    // draws the screen
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .areas(area);

        let [options, _, content] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(bottom);

        let options_block = Block::default()
            .borders(Borders::ALL)
            .title("List Screen")
            .style(style::Style::default().fg(style::Color::White));
        frame.render_widget(options_block, options);

        let content_block = Block::default()
            .borders(Borders::ALL)
            .title("Options")
            .style(style::Style::default().fg(style::Color::LightBlue));
        frame.render_widget(content_block, content);

        self.navbar.render(frame, top);
    }

    // returns an actiion based on the input that the app will act upon
    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match self.focus {
            Focus::NavBar => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('k') | KeyCode::Down => {
                            self.navbar.deselect();
                            self.focus = Focus::Content;
                        }
                        _ => {}
                    }
                } else {
                    return self.navbar.handle_input(key_event);
                }
            }
            _ => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        _ => {
                            self.focus = Focus::NavBar;
                            self.navbar.select();
                        }
                    }
                }
            }
        }
        None
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
    }
}
