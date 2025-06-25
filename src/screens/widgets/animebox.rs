use std::{fmt::format, sync::{Arc, Mutex}};

use ratatui::{
    buffer::Buffer, layout::{Alignment, Constraint, Direction, Layout, Margin, Rect}, style::{Color, Modifier, Style}, symbols, widgets::{Block, Borders, Clear, Padding, Paragraph, Widget}, Frame
};


use crate::{mal::models::anime::{fields::*, Anime}, utils::imageManager::ImageManager};
pub struct AnimeBox{
}

impl AnimeBox {
    pub fn render(anime: &Anime, image_manager: Arc<Mutex<ImageManager>>, frame: &mut Frame, area: Rect, highlight: bool) {
        if anime.id == 0 {
            let title = Paragraph::new("")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).padding(Padding::new(1, 1, 1, 1)));
            frame.render_widget(title, area);
            return;
        }

        let color = if highlight {Color::Yellow} else {Color::DarkGray};
        let title_text = anime.title.clone(); 

        let [left_part, right_part] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ])
            .areas(area);

        let title = Paragraph::new(title_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(color))
            .block(Block::default().padding(Padding::new(1, 1, 1, 1)));
        frame.render_widget(title, right_part);
        if highlight {
            frame.render_widget(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_set(symbols::border::ROUNDED),
                area,
            );

            let (box_set, box_border) = (
                symbols::border::Set {
                    bottom_left: symbols::line::HORIZONTAL_UP,
                    top_left: symbols::line::HORIZONTAL_DOWN,
                    ..symbols::border::PLAIN
                },
                Borders::ALL,
            );

            frame.render_widget(
                Block::new()
                    .borders(box_border)
                    .border_style(color)
                    .border_set(box_set),
                right_part,
            );
        } 
        else{
            frame.render_widget(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(color)
                    .border_set(symbols::border::ROUNDED),
                right_part,
            );
        }
        let image_area = left_part.inner(Margin::new(1, 1,));
        if let Ok(mut manager) = image_manager.try_lock() {
            manager.render_image(
                anime.id,
                frame,
                image_area
            );
        } 
    }
}
