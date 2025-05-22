use ratatui::{layout::{Alignment, Rect}, style::{Color, Style}, widgets::{Clear, Paragraph}, Frame};
use super::button::Button;


pub struct NavBar {
    pub selected_button: usize,
    pub buttons: Vec<String>,
}

impl NavBar {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            buttons: Vec::new(), 
        }
    }

    pub fn empty() -> Self {
        Self {
            selected_button: 0,
            buttons: vec![],
        }
    }

    pub fn add_button(&mut self, button: String) -> &mut Self {
        self.buttons.push(button);
        self
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let button_width = area.width / self.buttons.len() as u16;
        for (i, button) in self.buttons.iter().enumerate() {
            let x = area.x + (i as u16 * button_width);
            let button_area = Rect::new(x, area.y, button_width, area.height);

            f.render_widget(
                Paragraph::new(button.as_str())
                    .alignment(Alignment::Center),
                button_area
            );
        }
    }
}
