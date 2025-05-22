use super::widgets::animebox::AnimeBox;
use super::widgets::navbar::{self, NavBar};
use super::{screens::*, Screen};
use crate::{models::anime::Anime, ui::widgets::button::Button};
use crate::app::Action;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span, Text}, widgets::{ Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap}, Frame,
    symbols,
};
use crossterm::event::{KeyCode, KeyEvent};
use std::cmp::{max, min};
use std::os::unix::raw::blkcnt_t;


#[derive(Clone)]
pub struct OverviewScreen { 
    animes: Vec<Anime>,
    options: Vec<&'static str>,
    selected_button: usize,
}

impl OverviewScreen {
    pub fn new() -> Self {
        Self {
            animes: vec![
                Anime::empty(),
                Anime::empty(),
                Anime::empty(),
                Anime::empty(),
            ],

            options: vec![
                "Overview",
                "Seasons",
                "Lists",
                "Filters",
                "Proile",
            ],

            selected_button: 0,
        }
    }
}

impl Screen for OverviewScreen {
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);


        /* Splitting the screen:
        * which looks like this:
        * ╭────────╮
        * ╰────────╯
        * ╭─────╮╭─╮
        * ╰─────╯│ │
        * ╭─────╮│ │
        * │     ││ │
        * ╰─────╯╰─╯
        * */
        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(4),
                    Constraint::Percentage(100),
                ]
            )
            .areas(area);


        let [bottom_left, bottom_right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ]
            )
            .areas(bottom);

        let [bl_top, bl_bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Percentage(100),
                ]
            )
            .areas(bottom_left);


        /* Displayes the navbar:
        * which after looks like this:
        * ╭──┬──┬──╮
        * ╰──┴──┴──╯
        * ╭─────╮╭─╮
        * ╰─────╯│ │
        * ╭─────╮│ │ 
        * │     ││ │
        * ╰─────╯╰─╯
        * */

        let mut navbar = NavBar::new();
        for opt in self.options.iter() {
            navbar.add_button(opt.to_string());
        }
        navbar.render(frame, top);


        /* Displayes the bottom sections:
        * which after looks like this (ish):
        * ╭──┬──┬──╮
        * ╰──┴──┴──╯
        * ╭─────┬──╮
        * │     │  │
        * │     │  │ 
        * ╰─────┴──╯
        * */
        let (right_set, right_border) = 
        (
            symbols::border::Set {
                bottom_right: symbols::line::ROUNDED_BOTTOM_RIGHT,
                top_right: symbols::line::ROUNDED_TOP_RIGHT,
                ..symbols::border::PLAIN
            },
            Borders::RIGHT | Borders::BOTTOM | Borders::TOP
        );

        // bottom left top (blt)
        let (blt_set, blt_border) = 
        (
            symbols::border::Set {
                top_left: symbols::line::ROUNDED_TOP_LEFT,
                bottom_left: symbols::line::NORMAL.vertical_right,
                top_right: symbols::line::NORMAL.horizontal_down,
                bottom_right: symbols::line::NORMAL.vertical_left,
                ..symbols::border::PLAIN
            },
            Borders::ALL
        );

        let (blb_set, blb_border) = 
        (
            symbols::border::Set {
                bottom_left: symbols::line::ROUNDED_BOTTOM_LEFT,
                bottom_right: symbols::line::NORMAL.horizontal_up,
                ..symbols::border::PLAIN
            },
            Borders::LEFT | Borders::BOTTOM | Borders::RIGHT
        );


        let color = Style::default().fg(Color::Cyan);

        frame.render_widget(Block::new().border_set(right_set).borders(right_border).border_style(color), bottom_right);
        frame.render_widget(Block::new().border_set(blt_set).borders(blt_border).border_style(color), bl_top);
        frame.render_widget(Block::new().border_set(blb_set).borders(blb_border).border_style(color), bl_bottom);
    }

    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('j') => {
                if self.selected_button > 0 {
                    self.selected_button -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('k') => {
                if self.selected_button < self.options.len() - 1 {
                    self.selected_button += 1;
                }
            }
            KeyCode::Enter => {
                match self.selected_button {
                    0 => return Some(Action::SwitchScreen(PROFILE)),
                    1 => return Some(Action::SwitchScreen(SETTINGS)),
                    2 => return Some(Action::SwitchScreen(INFO)),
                    3 => return Some(Action::SwitchScreen(LAUNCH)),
                    _ => {}
                }
            }
            _ => {} 
        };
        None
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
    }
}
