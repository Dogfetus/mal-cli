use crate::app::Action;

use super::screens::*;
use super::widgets::navbar::NavBar;
use super::Screen;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::Frame;
use ratatui::widgets;
use ratatui::style;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    NavBar,
    Content,
}

#[derive(Clone)]
pub struct ProfileScreen {
    navbar: NavBar,
    focus: Focus,

}
impl ProfileScreen {
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

impl Screen for ProfileScreen {

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(widgets::Clear, area);

        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .areas(area);

        self.navbar.render(frame, top);

        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(32),
                Constraint::Percentage(68),
            ])
            .areas(bottom);

        let block = Block::new().borders(Borders::ALL)
            .style(style::Style::default().fg(style::Color::LightBlue));

        let [pfp, info, buttons] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
            ])
            .areas(left);


        frame.render_widget(block.clone(), left);
        frame.render_widget(block.clone(), right);
        frame.render_widget(block.clone(), pfp);
        frame.render_widget(block.clone(), info);
        frame.render_widget(block, buttons);
    }

    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {

        match self.focus {
            Focus::NavBar => {
                if let Some(action) = self.navbar.handle_input(key_event) {
                    return Some(action);
                }
            },
            Focus::Content => {
                match key_event.code {
                    _ => {
                        self.navbar.select();
                        self.focus = Focus::NavBar;
                    },
                }
            },
        }

        None
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
     } 

}

