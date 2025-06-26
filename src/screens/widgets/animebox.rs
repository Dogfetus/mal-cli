use std::{
    fmt::format,
    sync::{Arc, Mutex},
};

use ratatui::{
    buffer::Buffer, layout::{Alignment, Constraint, Direction, Layout, Margin, Rect}, style::{Color, Modifier, Style}, symbols, widgets::{Block, Borders, Clear, Padding, Paragraph, Widget, Wrap}, Frame
};

use crate::{
    mal::models::anime::{Anime, fields::*},
    utils::imageManager::ImageManager,
};
pub struct AnimeBox {}

impl AnimeBox {
    pub fn render(
        anime: &Anime,
        image_manager: &Arc<Mutex<ImageManager>>,
        frame: &mut Frame,
        area: Rect,
        highlight: bool,
    ) {
        if anime.id == 0 {
            let title = Paragraph::new("")
                .alignment(Alignment::Center)
                .style(Style::default().fg(if highlight {
                    Color::Yellow
                } else {
                    Color::DarkGray

                }))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .padding(Padding::new(1, 1, 1, 1)),
                );
            frame.render_widget(title, area);
            return;
        }

        let color = if highlight {
            Color::Yellow
        } else {
            if anime.my_list_status.status.is_empty() {
                Color::DarkGray
            }
            else {
                Color::Green
            }
        };

        let has_en_title = !anime.alternative_titles.en.is_empty();
        let title_text = if has_en_title {
            anime.alternative_titles.en.clone()
        } else {
            anime.title.clone()
        };

        let [left_part, right_part] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(area);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_style(color)
                .border_set(symbols::border::ROUNDED),
            area,
        );




        // title + split into info area
        let [title_area, info_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Fill(1)])
            .areas(area);

        let (info_set, info_borders) = (
            symbols::border::Set {
                top_right: symbols::line::VERTICAL_LEFT,
                top_left: symbols::line::VERTICAL_RIGHT,
                ..symbols::border::ROUNDED
            },
            Borders::ALL,
        );

        let info_block = Block::default()
            .borders(info_borders)
            .border_set(info_set)
            .style(Style::default().fg(color));
        frame.render_widget(info_block, info_area);

        // let title_area = Rect {
        //     x: right_part.x,
        //     y: right_part.y,
        //     width: right_part.width,
        //     height: title_area.height,
        // };

        let title = Paragraph::new(title_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .block(Block::default().padding(Padding::new(2, 2, 1, 0)));
        frame.render_widget(title, title_area);

        let [image_area, info_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(info_area);

        let image_area = image_area.inner(Margin::new(1, 1));
        ImageManager::render_image(&image_manager, anime.id, frame, image_area);

        let info_text = 
            "Score:\nType:\nEpisodes:\nStatus:\nAired:";

        let value_text = format!(
            "{}\n{}\n{}\n{}",
            anime.mean,
            anime.media_type,
            anime.num_episodes,
            anime.status,
        );
        let airing_text = format!(
            "{} -> {}",
            anime.start_date,
            anime.end_date
        );
        let user_stats_value_text = format!(
            "{}",
            anime.my_list_status.status,
        );

        let [info, value] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(info_area);

        let info_paragraph = Paragraph::new(info_text)
            .alignment(Alignment::Left)
            .style(Style::default().fg(color))
            .block(Block::default().padding(Padding::new(0, 0, 1, 1)));

        let value_paragraph = Paragraph::new(value_text)
            .alignment(Alignment::Left)
            .style(Style::default().fg(color))
            .block(Block::default().padding(Padding::new(0, 1, 1, 1)));

        let airing_paragraph = Paragraph::new(airing_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(color))
            .wrap(Wrap { trim: true })  
            .block(Block::default().padding(Padding::new(0, 2, 6, 1)));

        let [info_area, user_stats_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(2)])
            .areas(info_area);

        let user_stats_value_paragraph = Paragraph::new(user_stats_value_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(color))
            .block(Block::default().padding(Padding::new(0, 1, 0, 1)));

        frame.render_widget(info_paragraph, info);
        frame.render_widget(value_paragraph, value);
        frame.render_widget(airing_paragraph, info_area);
        frame.render_widget(user_stats_value_paragraph, user_stats_area);
    }
}
