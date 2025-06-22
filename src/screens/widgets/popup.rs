use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect}, 
    style::{Color, Style}, 
    symbols::{self, border}, 
    text::Line, 
    widgets::{Block, Borders, Clear, Paragraph}, 
    Frame
};
use crate::{app::Action, mal::models::anime::Anime, screens::{name_to_screen, screen_to_name}};


#[derive(Clone)]
pub struct Popup {
    pub anime: Anime,
    pub toggled: bool,
}

impl Popup {
    pub fn new() -> Self {
        Self {
            anime: Anime::empty(),
            toggled: false,
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
            },
            _ => None,
        }
    }
    pub fn render(&self, frame: &mut Frame) {
        if !self.toggled {
            return;
        }
        let area = frame.area();

        let [height, width] = [
            area.height * 7 / 10,
            area.width / 2,
        ];
        let popup_area = Rect::new(
            area.x + (area.width - width)/2,
            area.y + (area.height - height)/2,
            width,
            height,
        );
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .title("Anime Details")
            .style(Style::default().fg(Color::White));
        frame.render_widget(block.clone(), popup_area);

        let inner_area = block.inner(popup_area);
        let text = format!(
            "Title: {}\nStatus: {}\nEpisodes: {}\nScore: {}",
            self.anime.title, self.anime.status, self.anime.num_episodes, self.anime.mean
        );
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, inner_area);
    }

}

