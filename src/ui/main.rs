use ratatui::Frame;
use ratatui::widgets;
use ratatui::style;


#[allow(unused)]
pub fn draw(frame: &mut Frame, app: &crate::app::App) {

    let size = frame.area();
    let block = widgets::Block::default()
        .title("Main page:")
        .borders(widgets::Borders::ALL);
    let list = widgets::List::new(vec![
        widgets::ListItem::new("Anime 1"),
        widgets::ListItem::new("Anime 2"),
        widgets::ListItem::new("Anime 3"),
    ])
    .block(block)
    .highlight_style(style::Style::default().bg(style::Color::Blue));
    
    frame.render_widget(list, size);

}


    // this lets you devide the area into two parts 
    // use ratatui::layout::Constraint;
    // use ratatui::layout::Layout;
    // let test = Layout::vertical([Constraint::Percentage(10), Constraint::Percentage(90)]);
    // let [title, area]  = test.areas(frame.area());
