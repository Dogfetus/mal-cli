use ratatui::{
    layout::{Alignment, Rect}, 
    style::{Color, Style}, 
    widgets::{Block, Borders, Paragraph, Widget}, 
    Frame
};


#[derive(Debug, Clone)]
pub struct Button {
    text: String,
    width: u16,
    height: u16,
    offset_y: i16,
    offset_x: i16,
    is_selected: bool,
    is_centered_x: bool,
    is_centered_y: bool,
}

impl Button {
    pub fn new(text: &str) -> Self {
        Button {
            text: text.to_string(),
            width: 20,
            height: 3,
            offset_y: 0,
            offset_x: 0,
            is_selected: false,
            is_centered_x: false,
            is_centered_y: false,
        }
    }

    #[allow(dead_code)]
    pub fn offset(mut self, (offset_x, offset_y): (i16, i16)) -> Self {
        self.offset_y = offset_y;
        self.offset_x = offset_x;  
        self
    }

    #[allow(dead_code)]
    pub fn size(mut self, (width, height): (u16, u16)) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    #[allow(dead_code)]
    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    #[allow(dead_code)]
    pub fn center(mut self) -> Self {
        self.is_centered_x = true;
        self.is_centered_y = true;
        self
    }

    #[allow(dead_code)]
    pub fn center_x(mut self) -> Self {
        self.is_centered_x = true;
        self
    }

    #[allow(dead_code)]
    pub fn center_y(mut self) -> Self {
        self.is_centered_y = true;
        self
    }


    // WARNING: buttons still seem to crash the program when small screen
    pub fn render(self, f: &mut Frame, area: Rect) {
        let mut center_x = area.x;
        let mut center_y = area.y;

        if self.is_centered_x {
            center_x += (area.width - self.width) / 2;
        }
        if self.is_centered_y {
            center_y += (area.height - self.height) / 2;
        }

        let x_pos = if self.offset_x >= 0 {
            center_x.saturating_add(self.offset_x as u16)
        } else {
            center_x.saturating_sub((-self.offset_x) as u16)
        };
        let y_pos = if self.offset_y >= 0 {
            center_y.saturating_add(self.offset_y as u16)
        } else {
            center_y.saturating_sub((-self.offset_y) as u16)
        };

        if x_pos + self.width > area.x + area.width ||
           y_pos + self.height > area.y + area.height {
            return;
        }

        let button_area = Rect::new(
            x_pos, 
            y_pos, 
            self.width, 
            self.height);

        let color = if self.is_selected {
            Color::LightBlue
        } else {
            Color::Cyan
        };

        let button = Paragraph::new(self.text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);

        f.render_widget(button, button_area);
    }
}




// could also use this
impl Widget for Button {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.width < self.width || area.height < self.height {
            return;
        }

        let center_x = area.x + (area.width - self.width) / 2;
        let center_y = area.y + (area.height - self.height) / 2;

        let x_pos = if self.offset_x >= 0 {
            center_x.saturating_add(self.offset_x as u16)
        } else {
            center_x.saturating_sub((-self.offset_x) as u16)
        };
        let y_pos = if self.offset_y >= 0 {
            center_y.saturating_add(self.offset_y as u16)
        } else {
            center_y.saturating_sub((-self.offset_y) as u16)
        };

        if x_pos + self.width > area.x + area.width ||
           y_pos + self.height > area.y + area.height {
            return;
        }

        let button_area = Rect::new(
            x_pos, 
            y_pos, 
            self.width, 
            self.height);

        Paragraph::new(self.text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .render(button_area, buf)
    }
}
