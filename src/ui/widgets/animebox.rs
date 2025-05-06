use std::fmt::format;

use ratatui::{
    buffer::Buffer, layout::{Alignment, Constraint, Direction, Layout, Rect}, 
    style::{Color, Modifier, Style}, widgets::{Block, Borders, Clear, Paragraph, Widget}, Frame
};


use crate::models::anime::Anime;
pub struct AnimeBox<'a>{
    anime: &'a Anime,
    offset_x: u16,
    offset_y: u16,
    width: u16,
    height: u16,
    is_selected: bool,
    is_centered_x: bool,
    is_centered_y: bool,
}
impl<'a> AnimeBox<'a> {
    pub fn new(anime: &'a Anime) -> Self {
        Self {
            anime,
            offset_x: 0,
            offset_y: 0,
            width: 40,
            height: 18,
            is_selected: false,
            is_centered_x: false,
            is_centered_y: false,
        }
    }

    #[allow(dead_code)]
    pub fn offset(mut self, (offset_x, offset_y): (u16, u16)) -> Self {
        self.offset_y = offset_y;
        self.offset_x = offset_x;  
        self
    }

    pub fn offset_area(mut self, area: Rect) -> Self {
        self.offset_y = area.y;
        self.offset_x = area.x;
        self
    }

    #[allow(dead_code)]
    pub fn size(mut self, (width, height): (u16, u16)) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    #[allow(dead_code)]
    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    #[allow(dead_code)]
    pub fn center(mut self) -> Self {
        self.is_centered_x = true;
        self.is_centered_y = true;
        self
    }

    #[allow(dead_code)]
    pub fn center_x(mut self) -> Self {
        self.is_centered_x = true;
        self
    }

    #[allow(dead_code)]
    pub fn center_y(mut self) -> Self {
        self.is_centered_y = true;
        self
    }

    pub fn render(&self, frame: &mut Frame) {
        let anime_box_rect = Rect::new(
            self.offset_x,
            self.offset_y,
            self.width,
            self.height
        );

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

        let title_bar = Paragraph::new(self.anime.title.clone())
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Cyan)));

        let image_placeholder = Paragraph::new("IMG")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(Color::Cyan)));

        let info = vec![
            format!("Score: {}", self.anime.mean),
            format!("Status: {}", self.anime.status),
            "".to_string(),
            format!("Synopsis:{}", self.anime.synopsis),
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

        frame.render_widget(title_bar, anime_box_layout[0]);
        frame.render_widget(image_placeholder, content_layout[0]);    
        frame.render_widget(info_section, content_layout[1]);
        frame.render_widget(bottom_section, anime_box_layout[2]);
    }
}
