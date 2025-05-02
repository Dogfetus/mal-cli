use std::{cmp::{max, min}, sync::{mpsc::Sender, Mutex}};
use crate::{app::Event, ui::widgets::button::Button};
use crossterm::event::{KeyCode, KeyEvent};
use std::sync::atomic::AtomicBool;
use super::{screens::*, Screen};
use std::thread::JoinHandle;
use crate::mal::init_oauth;
use crate::app::Action;
use std::sync::Arc;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect}, 
    widgets::{ Block, Borders, Clear, Paragraph, Wrap}, 
    style::{Color, Modifier, Style}, 
    text::{Line, Span, Text}, 
    Frame 
};


#[derive(Clone)]
pub struct LoginScreen { 
    selected_button: usize,
    buttons: Vec<&'static str>,
    login_url: Arc<Mutex<String>>,
    is_signed_in: bool,
}

impl LoginScreen {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            buttons: vec![
                "Copy",
                "Paste",
                "Back",
            ],
            login_url: Arc::new(Mutex::new(String::new())),
            is_signed_in: false,
        }
    }
}

impl Screen for LoginScreen {
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        frame.render_widget(Clear, area);

        let page_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);


        let header_text = vec![
            "        ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⡀⠠⠀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⠀⢀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
        ];

        let alpha = Paragraph::new(header_text.join("\n"))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

        frame.render_widget(alpha, page_chunk[0]);

        let text_field_area = Rect::new(
            page_chunk[1].x + min(page_chunk[1].width / 2 - 25, page_chunk[1].width / 4),
            page_chunk[1].y + 2,
            max(page_chunk[1].width / 2, 50),
            3);

        // if self.login_url.is_empty() {
        //     self.login_url = init_oauth().0;
        // }

        let url_field = Paragraph::new(self.login_url.lock().unwrap().clone())
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);

        frame.render_widget(url_field, text_field_area);

        for (i, button) in self.buttons.iter().enumerate() {
            Button::new(button)
                .offset((0, -3 + (i as i16 * 3)))
                .center()
                .selected(i == self.selected_button)
                .render(frame, page_chunk[1]);
        }

    }

    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('j') => {
                if self.selected_button > 0 {
                    self.selected_button -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('k') => {
                if self.selected_button < self.buttons.len() - 1 {
                    self.selected_button += 1;
                }
            }
            KeyCode::Enter => {
                match self.selected_button {
                    _ => { return Some(Action::SwitchScreen(LAUNCH)); }
                }
            }
            _ => {} 
        };
        None
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
    }

    fn background(&self, sx: Sender<Event>, stop: Arc<AtomicBool>) -> Option<JoinHandle<()>> {

        let sx = sx.clone();
        let stop = stop.clone();
        let login_url = self.login_url.clone();

        Some(std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));

            let url_to_print = "this is a test url test url that will be displayed for the user to copy and paste into their browser yehaw!";
            for i in 0..url_to_print.len()+1 {
                std::thread::sleep(std::time::Duration::from_millis(8));
                {
                    let mut url = login_url.lock().unwrap();
                    *url = url_to_print[0..i].to_string();
                }
                sx.send(Event::Rerender).unwrap();
            }
        }))
    }
}
