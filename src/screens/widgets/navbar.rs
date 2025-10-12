use crate::{
    app::Action,
    config::Config,
    screens::{name_to_screen, screen_to_name},
};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect},
    style::Style,
    symbols,
    text::Line,
    widgets::{Block, Borders},
};

#[derive(Clone)]
pub struct NavBar {
    old_button: usize,
    selected_button: usize,
    options: Vec<&'static str>,
    is_selected: bool,
    option_areas: Vec<Rect>,
}

impl NavBar {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            old_button: 0,
            options: Vec::new(),
            is_selected: false,
            option_areas: Vec::new(),
        }
    }

    pub fn is_selected(&self) -> bool {
        self.is_selected
    }

    pub fn select(&mut self) -> &Self {
        self.is_selected = true;
        self
    }

    pub fn deselect(&mut self) -> &Self {
        self.selected_button = self.old_button;
        self.is_selected = false;
        self
    }

    pub fn add_screen(mut self, screen: &'static str) -> Self {
        self.options.push(screen_to_name(screen));
        self
    }

    pub fn handle_keyboard(&mut self, key_event: KeyEvent) -> Option<Action> {
        if (key_event.code == KeyCode::Char('k') || key_event.code == KeyCode::Down)
            && key_event
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            self.deselect();
            return Some(Action::NavbarSelect(false));
        }

        match key_event.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.selected_button > 0 {
                    self.selected_button = self.selected_button.saturating_sub(1);
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.selected_button < self.options.len() - 1 {
                    self.selected_button += 1;
                }
            }
            KeyCode::Enter => {
                self.old_button = self.selected_button;
                let screen_name = self.options[self.selected_button];
                return Some(Action::SwitchScreen(name_to_screen(screen_name)));
            }
            _ => {}
        }
        None
    }

    pub fn handle_mouse(&mut self, mouse_event: MouseEvent) -> Option<Action> {
        if mouse_event.row > 2 {
            self.deselect();
            return None;
        }

        if self.option_areas.is_empty() {
            return None;
        }

        for (i, area) in self.option_areas.iter().enumerate() {
            let pos = Position::new(mouse_event.column, mouse_event.row);
            if area.inner(Margin::new(1, 0)).contains(pos) {
                self.selected_button = i;
                break;
            }
        }

        if let crossterm::event::MouseEventKind::Down(_) = mouse_event.kind {
            self.old_button = self.selected_button;
            let screen_name = self.options[self.selected_button];
            return Some(Action::SwitchScreen(name_to_screen(screen_name)));
        }

        None
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let constraints: Vec<Constraint> = (0..self.options.len())
            .map(|_| Constraint::Ratio(1, self.options.len() as u32))
            .collect();

        let option_rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        self.option_areas = option_rects.to_vec();

        let overall_color = if self.is_selected {
            Style::default().fg(Config::global().theme.highlight)
        } else {
            Style::default().fg(Config::global().theme.primary)
        };

        for (i, opt) in self.options.iter().enumerate() {
            let option_rect = option_rects[i];

            let (border_set, borders) = match i {
                0 => (
                    symbols::border::Set {
                        bottom_left: symbols::line::ROUNDED_BOTTOM_LEFT,
                        top_left: symbols::line::ROUNDED_TOP_LEFT,
                        ..symbols::border::PLAIN
                    },
                    Borders::LEFT | Borders::TOP | Borders::BOTTOM,
                ),
                i if i == self.options.len() - 1 => (
                    symbols::border::Set {
                        top_right: symbols::line::ROUNDED_TOP_RIGHT,
                        top_left: symbols::line::NORMAL.horizontal_down,
                        bottom_left: symbols::line::NORMAL.horizontal_up,
                        bottom_right: symbols::line::ROUNDED_BOTTOM_RIGHT,
                        ..symbols::border::PLAIN
                    },
                    Borders::ALL,
                ),
                _ => (
                    symbols::border::Set {
                        bottom_left: symbols::line::NORMAL.horizontal_up,
                        top_left: symbols::line::NORMAL.horizontal_down,
                        ..symbols::border::PLAIN
                    },
                    Borders::LEFT | Borders::TOP | Borders::BOTTOM,
                ),
            };

            let option = Block::new()
                .border_set(border_set)
                .borders(borders)
                .border_style(overall_color);
            frame.render_widget(&option, option_rect);

            let inner = option.inner(option_rect);
            let text_y = inner.y + (inner.height) / 2;
            let centered_area = Rect::new(inner.x, text_y, inner.width, 1);
            let style = if self.is_selected && i == self.selected_button {
                Style::default().fg(Config::global().theme.second_highlight)
            } else if i == self.old_button {
                Style::default().fg(Config::global().theme.highlight)
            } else {
                Style::default().fg(Config::global().theme.text)
            };

            frame.render_widget(
                Line::from(opt.to_string())
                    .alignment(Alignment::Center)
                    .style(style),
                centered_area,
            );
        }
    }
}
