use ratatui::{layout::{Alignment, Constraint, Direction, Layout, Margin, Rect}, style::{Color, Style}, widgets::Paragraph, Frame};

#[derive(Clone)]
struct InfoItem {
    label: String,
    value: String,
    show_hash: bool,
}

impl InfoItem {
    fn new(label: &str, value: String, show_hash: bool) -> Self {
        Self {
            label: label.to_string(),
            value,
            show_hash,
        }
    }
}

pub struct InfoBox {
    rows: Vec<Vec<InfoItem>>,
    current_row: Vec<InfoItem>,
}

impl InfoBox {
    pub fn new() -> Self {
        Self { 
            rows: Vec::new(),
            current_row: Vec::new(),
        }
    }

    pub fn add_item(mut self, label: &str, value: String, show_hash: bool) -> Self {
        self.current_row.push(InfoItem::new(label, value, show_hash));
        self
    }

    pub fn add_ranked_item(self, label: &str, value: String) -> Self {
        self.add_item(label, value, true)
    }

    pub fn add_text_item(self, label: &str, value: String) -> Self {
        self.add_item(label, value, false)
    }

    pub fn add_row(mut self) -> Self {
        if !self.current_row.is_empty() {
            self.rows.push(std::mem::take(&mut self.current_row));
        }
        self
    }

        pub fn render(mut self, frame: &mut Frame, area: Rect, margin: Margin, primary_color: Color) -> Rect {
        // Add the current row if it has items
        if !self.current_row.is_empty() {
            self.rows.push(self.current_row);
        }

        if self.rows.is_empty() {
            return area; // Return unchanged area if no content
        }

        // Calculate required height (2 units per row)
        let required_height = self.rows.len() as u16 * 2;
        
        // Create our own layout within the given area
        let [info_section, remaining] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(required_height),
                Constraint::Fill(1),
            ])
            .areas(area);

        // Create vertical layout for rows within our section
        let row_constraints: Vec<Constraint> = vec![Constraint::Length(2); self.rows.len()];
        let row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(info_section);

        // Render each row
        for (row_index, row) in self.rows.iter().enumerate() {
            if row_index < row_areas.len() {
                render_info_row(frame, row_areas[row_index], row, margin, primary_color);
            }
        }

        remaining // Return the remaining area for other widgets
    }
}

fn render_info_row(
    frame: &mut Frame,
    area: Rect,
    items: &[InfoItem],
    margin: Margin,
    primary_color: Color,
) {
    let info_sides = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Fill(1); items.len()])
        .split(area.inner(margin));

    for (i, item) in items.iter().enumerate() {
        if i >= info_sides.len() {
            break;
        }

        let info_area = info_sides[i];
        let [left, right, _spare] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(item.label.len() as u16 + 2),
                Constraint::Length(item.value.len() as u16 + if item.show_hash { 1 } else { 0 }),
                Constraint::Fill(1),
            ])
            .areas(info_area);

        // Render label
        let text_paragraph = Paragraph::new(item.label.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(primary_color));
        frame.render_widget(text_paragraph, left);

        // Render value
        let formatted_value = if item.show_hash {
            format!("#{}", item.value)
        } else {
            item.value.clone()
        };
        
        let value_paragraph = Paragraph::new(formatted_value)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        frame.render_widget(value_paragraph, right);
    }
}
