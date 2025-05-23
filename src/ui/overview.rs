use super::widgets::navbar::NavBar;
use super::{screens::*, Screen};
use crate::models::anime::Anime;
use crate::app::Action;
use ratatui::widgets::{Padding, Paragraph};
use ratatui::{
    layout::{Constraint, Direction, Layout}, style::{Color, Style}, widgets::{Block,  Borders, Clear}, Frame,
    symbols,
};
use crossterm::event::{KeyCode, KeyEvent};


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


        let color = Style::default().fg(Color::DarkGray);

        frame.render_widget(Block::new().border_set(right_set).borders(right_border).border_style(color), bottom_right);
        frame.render_widget(Block::new().border_set(blt_set).borders(blt_border).border_style(color), bl_top);
        frame.render_widget(Block::new().border_set(blb_set).borders(blb_border).border_style(color), bl_bottom);



        /* then create grid for animes:
        *  margin to keep grid from ruining area:
        * this part:
        * ╭─────╮
        * ╰─────╯
        * ╭─────╮
        * │     │
        * ╰─────╯
        */
        let [blb_left, blb_middle, blb_right] = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(1)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ]
            )
            .areas(bl_bottom);
        for column in [blb_left, blb_middle, blb_right] {
            let [top, middle, bottom] = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ]
                )
                .areas(column);
            for area in [top, middle, bottom] {
                // the anime box should go here:
                frame.render_widget(Block::new().borders(Borders::ALL).border_style(color), area);
            }
        }



        /* render right side with anime data:
        * this part:
        * ╭─╮
        * │ │
        * │ │
        * │ │
        * ╰─╯
        */

        let [bottom_right, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ]
            )
            .areas(bottom_right);

        let [top, middle, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .vertical_margin(1)
            .constraints(
                [
                    Constraint::Length(7),
                    Constraint::Percentage(55),
                    Constraint::Percentage(45),
                ]
            )
            .areas(bottom_right);

        let example_title = ["Fire Force Season 3", "Enen no Shouboutai: San no ShouEnen no Shouboutai: San no ShouEnen no Shouboutai: San no ShouEnen no Shouboutai: San no Shou"];


        let title = Paragraph::new(format!("English:\n{}\n\nJapanese:\n{}", example_title[0], example_title[1]))
            .block(Block::default().padding(Padding::new(1, 1, 1, 1)));
        frame.render_widget(title, top);

        let details = ["Type:", "Episodes:", "Status:", "Aired:", "Producers:", "Genres:", "Duration:", "Rating:", "Score:", "Ranked:", "Popularity:", "Members:", "Favorites:", "Studios"];

        if middle.width > 50 {
            let [rigth, left] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ]
                )
                .areas(middle);
            for (i, detail) in details.iter().enumerate() {
                let text = Paragraph::new(format!("{} {}\n", detail, "test"))
                    .block(Block::default().padding(Padding::new(1, 2, 1, 1)));
                if i % 2 == 0 {
                    frame.render_widget(text, left);
                }
                else{
                    frame.render_widget(text, rigth);
                }
            }
        }
        else{
            let details = Paragraph::new(format!("width: {}", middle.width))
                .block(Block::default().padding(Padding::new(1, 1, 1, 1)).borders(Borders::TOP).padding(Padding::new(1, 2, 1, 1)));
            frame.render_widget(details, middle);
        }


        // TODO: this might break the sides (add margin? might not if just text)
        frame.render_widget(Block::new().borders(Borders::ALL).border_style(color), bottom);
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
