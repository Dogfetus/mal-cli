// use ratatui::{backend::CrosstermBackend, Terminal, Frame};
// use ratatui::text::{Text, Line, Span};
// use ratatui_image::{picker::Picker, StatefulImage, protocol::StatefulProtocol};
// use ratatui::widgets::{Paragraph, Block};
// use ratatui::layout::{Layout, Constraint, Direction};
//
//
//
// use anyhow::Result;
// use crossterm::event::{self, Event, KeyCode};
// use ratatui::DefaultTerminal;
// mod mal;
// mod models;
// use models::typed_example;
// use models::Person;
// use mal::test;
//

// struct App {
//     image: StatefulProtocol,
//     data: Vec<Person>,
//     scroll: u16,
// }

use std::io;
mod ui;
use ui::App;


fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();


    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    // let result = run(terminal);
    ratatui::restore();
    app_result

}


// fn run(mut terminal: DefaultTerminal) -> Result<()> {
//     let picker = Picker::from_query_stdio()?;
//
//     // Load an image with the image crate.
//     let dyn_img = image::ImageReader::open("./assets/Untitled.png")?.decode()?;
//
//     // Create the Protocol which will be used by the widget.
//     let image = picker.new_resize_protocol(dyn_img);  
//
//     let test = test()?;
//     let data = typed_example(&test)?;
//
//     let mut app = App { image, data, scroll: 0 };
//
//     loop {
//         terminal.draw(|f| render(f, &mut app))?;
//         if let Event::Key(key) = event::read()? {
//             match key.code {
//                 KeyCode::Up => {
//                     app.scroll = app.scroll.saturating_sub(1);
//                 }
//                 KeyCode::Down => {
//                     app.scroll += 1;
//                 }
//                 KeyCode::Esc => {
//                     app.image.last_encoding_result().unwrap()?;
//                     break Ok(());
//                 }
//                 _ => {}
//             }
//         }
//     }
// }
//
//
// fn render(f: &mut Frame, app: &mut App) {
//     let chunks = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Percentage(50),
//             Constraint::Percentage(50),
//         ])
//         .split(f.area());
//
//     let lines: Vec<Line> = app.data.iter()
//         .map(|person| Line::from(Span::raw(&person.name)))
//         .collect();
//
//     let paragraph = Paragraph::new(Text::from(lines))
//         .block(Block::default().title("Greeting"))
//         .scroll((app.scroll, 0));
//     f.render_widget(paragraph, chunks[1]);
//
//     let image = StatefulImage::default();
//     f.render_stateful_widget(image, chunks[0], &mut app.image);
//
// }
