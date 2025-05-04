use ratatui::{
    buffer::Buffer, layout::{Alignment, Rect}, style::{Color, Style}, widgets::{Block, Borders, Clear, Paragraph, Widget}, Frame
};

use crate::models::anime::Anime;
struct AnimeBox{
    anime: Anime,
    offset_x: i16,
    offset_y: i16,
    width: u16,
    height: u16,
    is_selected: bool,
    is_centered_x: bool,
    is_centered_y: bool,
}
impl AnimeBox {
    pub fn new(anime: Anime) -> Self {
        Self {
            anime,
            offset_x: 0,
            offset_y: 0,
            width: 20,
            height: 3,
            is_selected: false,
            is_centered_x: false,
            is_centered_y: false,
        }
    }

    #[allow(dead_code)]
    pub fn offset(mut self, (offset_x, offset_y): (i16, i16)) -> Self {
        self.offset_y = offset_y;
        self.offset_x = offset_x;  
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
        let area = frame.area();
        let mut anime_area = Rect::new(
            area.x + self.offset_x as u16,
            area.y + self.offset_y as u16,
            self.width,
            self.height,
        );

        if self.is_centered_x {
            anime_area.x = (area.width - self.width) / 2;
        }
        if self.is_centered_y {
            anime_area.y = (area.height - self.height) / 2;
        }
    }
}

impl Widget for AnimeBox {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::default().bg(Color::Black));
    }
}
