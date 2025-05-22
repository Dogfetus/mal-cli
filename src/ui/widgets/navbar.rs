use ratatui::{layout::{Alignment, Rect}, style::{Color, Style}, symbols, text::Line, widgets::{Block, Borders, Clear, Paragraph}, Frame};


#[derive(Clone)]
pub struct NavBar {
    pub selected_button: usize,
    pub options: Vec<String>,  
}

impl NavBar {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            options: Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            selected_button: 0,
            options: Vec::new(),
        }
    }

    pub fn add_button(&mut self, button: String) -> &mut Self {
        self.options.push(button);
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let option_width = area.width / self.options.len() as u16;

        for (i, opt) in self.options.iter().enumerate() {
            let option_rect = Rect::new(area.x + (i as u16 * option_width), area.y, option_width, area.height);

            let (border_set, borders) = match i {
                0 => (
                    symbols::border::Set {
                        bottom_left: symbols::line::ROUNDED_BOTTOM_LEFT,
                        top_left: symbols::line::ROUNDED_TOP_LEFT,
                        ..symbols::border::PLAIN
                    },
                    Borders::LEFT | Borders::TOP | Borders::BOTTOM
                ),
                i if i == self.options.len() - 1 => (
                    symbols::border::Set {
                        top_right: symbols::line::ROUNDED_TOP_RIGHT,
                        top_left: symbols::line::NORMAL.horizontal_down,
                        bottom_left: symbols::line::NORMAL.horizontal_up,
                        bottom_right: symbols::line::ROUNDED_BOTTOM_RIGHT,
                        ..symbols::border::PLAIN
                    },
                    Borders::ALL
                ),
                _ => (
                    symbols::border::Set {
                        bottom_left: symbols::line::NORMAL.horizontal_up,
                        top_left: symbols::line::NORMAL.horizontal_down,
                        ..symbols::border::PLAIN
                    },
                    Borders::LEFT | Borders::TOP | Borders::BOTTOM
                ),
            };

            let option = Block::new().border_set(border_set).borders(borders)
                .border_style(Style::default().fg(Color::Cyan));
            frame.render_widget(&option, option_rect);

            let inner = option.inner(option_rect);
            let text_y = inner.y + (inner.height) / 2;
            let centered_area = Rect::new(inner.x, text_y, inner.width, 1);

            frame.render_widget(
                Line::from(opt.to_string()).alignment(Alignment::Center),
                centered_area
            );
        }
    }
}
