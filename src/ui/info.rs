use super::Screen;

use ratatui::Frame;
use ratatui::widgets;
use ratatui::style;


pub struct InfoPage {}

impl Screen for InfoPage {

    #[allow(unused)]
    fn draw(&self, frame: &mut Frame, app: &crate::app::App) {
        let size = frame.area();
        let block = widgets::Block::default()
            .title("Info page:")
            .borders(widgets::Borders::ALL);
        let list = widgets::List::new(vec![
            widgets::ListItem::new("Anime 2"),
        ])
        .block(block)
        .highlight_style(style::Style::default().bg(style::Color::Blue));
        
        frame.render_widget(list, size);

    }
}
