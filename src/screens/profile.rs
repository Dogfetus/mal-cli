use crate::app::Action;

use super::screens::*;
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

#[derive(Clone)]
pub struct ProfileScreen {
    selected_button: usize,
    buttons: Vec<&'static str>,
}
impl ProfileScreen {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            buttons: vec![
                "Back",
                "Exit",
            ],
        }
    }
}

impl Screen for ProfileScreen {

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(widgets::Clear, area);

        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(32),
                Constraint::Percentage(68),
            ])
            .areas(area);

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
            KeyCode::Enter => return Some(Action::SwitchScreen(OVERVIEW)),
            _ => {},
        }
        None
    }

    fn should_store(&self) -> bool {
        false
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
     } 

}

