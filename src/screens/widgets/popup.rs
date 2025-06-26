use std::sync::{Arc, Mutex};

use crate::{
    app::Action,
    mal::models::anime::Anime, utils::imageManager::ImageManager,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect}, style::{Color, Style}, symbols::{self, border}, text::Line, widgets::{Block, Borders, Clear, Paragraph}, Frame
};

#[derive(Clone)]
pub struct Popup {
    pub anime: Anime,
    pub toggled: bool,
    pub buttons: Vec<&'static str>,
}

impl Popup {
    pub fn new() -> Self {
        Self {
            anime: Anime::empty(),
            toggled: false,
            buttons: vec!["Play", "Add to List", "Add to Favorites", "Rate", "Share"],
        }
    }

    pub fn set_anime(&mut self, anime: Anime) -> &Self {
        self.anime = anime;
        self
    }

    pub fn toggle(&mut self) -> &Self {
        self.toggled = !self.toggled;
        self
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Char('q') => {
                self.toggled = false;
                None
            }
            _ => None,
        }
    }

    pub fn render(&self, image_manager: &Arc<Mutex<ImageManager>> , frame: &mut Frame) {
        if !self.toggled {
            return;
        }
        let area = frame.area();

        let [height, width] = [area.height * 8 / 10, area.width * 7 / 10];
        let popup_area = Rect::new(
            area.x + (area.width - width) / 2,
            area.y + (area.height - height) / 2,
            width,
            height,
        );
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .style(Style::default().fg(Color::White));
        frame.render_widget(block.clone(), popup_area);

        let [_, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .areas(popup_area);
        let [_, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(right);

        let (right_set, right_border) = (
            symbols::border::Set {
                bottom_left: symbols::line::ROUNDED_BOTTOM_RIGHT,
                top_right: symbols::line::ROUNDED_BOTTOM_RIGHT,
                ..symbols::border::ROUNDED
            },
            Borders::ALL,
        );

        let right_block = Block::default()
            .borders(right_border)
            .border_set(right_set)
            .style(Style::default().fg(Color::White));

        let bottom_area = Rect::new(
            bottom.x + 1,
            bottom.y + 1,
            bottom.width - 1,
            bottom.height - 1, // Leave space for the border
        );

        frame.render_widget(Clear, bottom);
        frame.render_widget(right_block, bottom);

        let button_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Ratio(1, self.buttons.len() as u32);
                self.buttons.len()
            ])
            .split(bottom_area);

        for (button, &area) in self.buttons.iter().zip(button_areas.iter()) {
            let paragraph = Paragraph::new(button.to_string())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED),
                )
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White));
            frame.render_widget(paragraph, area);
        }
        ImageManager::render_image(
            image_manager,
            self.anime.id,
            frame,
            popup_area.inner(Margin::new(1, 1)),
        );

    }
}


#[derive(Clone)]
pub struct SeasonPopup{
    pub toggled: bool,
    pub season: String,
    pub year: u32,
}
