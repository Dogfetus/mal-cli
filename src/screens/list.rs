use crate::{app::Action, screens::Screen};
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use crossterm::event::KeyEvent;
use crossterm::event::KeyCode;
use ratatui::widgets::Borders;
use ratatui::widgets::Block;
use ratatui::layout::Layout;
use ratatui::style;
use ratatui::widgets::Clear;
use ratatui::Frame;


#[derive(Clone)]
pub struct ListScreen {
    pub selected_button: usize,
    pub buttons: Vec<&'static str>,
}

impl ListScreen {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            buttons: vec![
                "Back",
                "Exit",
            ],
        }
    }
}

impl Screen for ListScreen {

    // draws the screen
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .areas(area);
        let block = Block::new().borders(Borders::ALL)
            .style(style::Style::default().fg(style::Color::Cyan));
        frame.render_widget(block.clone(), left);
        frame.render_widget(block, right);
    }


    // returns an actiion based on the input that the app will act upon
    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('j') => {}
            KeyCode::Down | KeyCode::Char('k') => {}
            KeyCode::Left | KeyCode::Char('h') => {}
            KeyCode::Right | KeyCode::Char('l') => {}
            KeyCode::Enter => {}
            _ => {} 
        };
        None
    }


    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
    }

    // -> the should store is only needed if the state of the screen should not be preserved when
    // swithcing screens if this is set to false the clone_box implementation is not needed
    // fn should_store(&self) -> bool {
    // false 
    // }
}


