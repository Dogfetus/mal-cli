use std::fmt::format;

use ratatui::{
    buffer::Buffer, layout::{Alignment, Constraint, Direction, Layout, Rect}, 
    style::{Color, Modifier, Style}, widgets::{Block, Borders, Clear, Paragraph, Widget}, Frame
};


use crate::mal::models::anime::{fields::*, Anime};
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
    }
}
