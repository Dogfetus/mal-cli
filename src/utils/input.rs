use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Position,
};

#[derive(Debug, Clone)]
pub struct Input {
    empty: bool,

    value: String,
    placeholder: Option<String>,

    max_length: Option<usize>,
    cursor: u16,
}

impl Input {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: None,
            max_length: None,
            cursor: 0,
            empty: true,
        }
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        let placeholder = placeholder.to_string();
        self.placeholder = Some(placeholder.clone());
        self.value = placeholder;
        self
    }

    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }

    pub fn render_cursor(&self, frame: &mut Frame, x: u16, y: u16) {
        frame.set_cursor_position(Position::new(x + self.cursor, y));
    }

    pub fn hide_cursor(&self, frame: &mut Frame) {
        frame.set_cursor_position(Position::new(u16::MAX, u16::MAX));
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<String> {
        if event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            match event.code {
                KeyCode::Char('h') => {
                    if self.cursor > 0 {
                        let cursor_pos = self.cursor as usize;
                        let chars: Vec<char> = self.value.chars().collect();

                        if cursor_pos <= chars.len() {
                            let mut start = cursor_pos;

                            while start > 0 && chars[start - 1].is_whitespace() {
                                start -= 1;
                            }

                            while start > 0 && !chars[start - 1].is_whitespace() {
                                start -= 1;
                            }

                            let start_byte = chars[..start].iter().map(|c| c.len_utf8()).sum::<usize>();
                            let end_byte = chars[..cursor_pos].iter().map(|c| c.len_utf8()).sum::<usize>();

                            self.value.replace_range(start_byte..end_byte, "");
                            self.cursor = start as u16;
                            if self.value.is_empty() {
                                self.empty = true;
                                self.value = self.placeholder.clone().unwrap_or_default();
                            }
                        }
                    }
                }
                //TODO: consider adding copy paste functionality
                _ => return None,
            }
        }

        else{
            match event.code {
                KeyCode::Char(c) if self.max_length.map_or(true, |l| self.value.len() < l) => {
                    if self.empty{
                        self.empty = false;
                        // in case of placeholder
                        self.value.clear();
                        self.cursor = 0;
                    }
                    self.value.insert(self.cursor as usize, c);
                    self.cursor += 1;
                }
                KeyCode::Backspace if self.cursor > 0 => {
                    self.value.remove((self.cursor - 1) as usize);
                    self.cursor -= 1;
                    if self.value.is_empty() {
                        self.empty = true;
                        self.value = self.placeholder.clone().unwrap_or_default();
                    }
                }
                KeyCode::Left if self.cursor > 0 => {
                    if !self.empty {
                        self.cursor -= 1;
                    }
                }
                KeyCode::Right if (self.cursor as usize) < self.value.len() => {
                    if !self.empty { 
                        self.cursor += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char('\n') => {
                    if !self.empty {
                        return Some(self.value.clone());
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
