use ratatui::{
    style::Style,
    widgets::{Block, Borders, Paragraph, Clear},
    Frame, 
};
use ratatui::style::Color;
use ratatui::layout::{Constraint, Direction, Layout, Alignment, Rect};


fn create_button<'a>(
    frame: &mut Frame<'a>,
    parent: Rect,
    text: &'a str,
    width: u16,
    height: u16,
    offset_y: i16,
) {
    if parent.width < width || parent.height < height {
        return;
    }

    let center_y = parent.y + (parent.height - height) / 2;
    let y_pos = if offset_y >= 0 {
        center_y.saturating_add(offset_y as u16)
    } else {
        center_y.saturating_sub((-offset_y) as u16)
    };

    if y_pos + height > parent.y + parent.height {
        return;
    }

    let button = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

    let button_area = Rect::new(
        parent.x + (parent.width - width) / 2,
        parent.y + (parent.height - height) / 2 + offset_y as u16, 
        width,
        height
    );
    frame.render_widget(button, button_area);
}


#[allow(unused)]
pub fn draw(frame: &mut Frame, app: &crate::app::App) {
    let area = frame.area();

    frame.render_widget(Clear, area);

    let page_chunk = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(area);

    let centeded_chunk = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Fill(1),
        Constraint::Percentage(30),
    ])
    .split(page_chunk[0]);


    let header_text = vec![
        " ███╗   ███╗ █████╗ ██╗                 ██████╗██╗     ██╗ ",
        " ████╗ ████║██╔══██╗██║                ██╔════╝██║     ██║ ",
        " ██╔████╔██║███████║██║      ███████╗  ██║     ██║     ██║ ",
        " ██║╚██╔╝██║██╔══██║██║      ╚══════╝  ██║     ██║     ██║ ",
        " ██║ ╚═╝ ██║██║  ██║███████╗           ╚██████╗███████╗██║ ",
        " ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝            ╚═════╝╚══════╝╚═╝ ",
    ];

    let alpha = Paragraph::new(header_text.join("\n"))
    .style(Style::default().fg(Color::Cyan))
    .alignment(Alignment::Center);

    frame.render_widget(alpha, centeded_chunk[1]);

    let width = 20;
    let height = 3;

    create_button(frame, area, "Browse", width, height, 7);
    create_button(frame, area, "Log in", width, height, 10);
    create_button(frame, area, "Exit", width, height, 13);
}

