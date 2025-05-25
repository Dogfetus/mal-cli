use crate::app::Action;

use super::screens::*;
use super::widgets::button::Button;
use super::Screen;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::widgets;
use ratatui::style;

#[derive(Clone)]
pub struct InfoScreen {
    selected_button: usize,
    buttons: Vec<&'static str>,
}
impl InfoScreen {
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

impl Screen for InfoScreen {

    fn draw(&self, frame: &mut Frame) {
        let size = frame.area();
        let block = widgets::Block::default()
            .title("Info page:")
            .borders(widgets::Borders::ALL);
        let list = widgets::List::new(vec![
            widgets::ListItem::new("Anime 2"),
        ])
        .block(block)
        .highlight_style(style::Style::default().bg(style::Color::Blue));
        frame.render_widget(list, size);


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
