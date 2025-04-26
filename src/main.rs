mod app;
mod ui;
mod mal;
mod models;
mod controller;



use crate::app::App;
use anyhow::Result;



#[tokio::main]
async fn main() -> Result<()> {
    // load .env


    // start the terminal view
    let mut terminal = ratatui::init();
    let mut app = App::new();
    app.run(&mut terminal)?;
    ratatui::restore();



    // mal::init_oauth();


    Ok(())

}






























// use std::io::{self, BufReader}; // Add this import
// use std::time::{Duration, Instant};
// use ratatui::{
//     backend::CrosstermBackend,
//     Terminal, Frame,
//     layout::{Layout, Constraint, Direction},
// };
// use ratatui_image::{picker::Picker, StatefulImage, protocol::StatefulProtocol};
// use crossterm::{
//     event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
//     execute,
//     terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
// };
// use image::{ImageDecoder, AnimationDecoder, codecs::gif::GifDecoder};
// use std::fs::File;
//
// struct App {
//     image: StatefulProtocol,
//     frames: Vec<image::DynamicImage>,
//     current_frame: usize,
//     frame_duration: Duration,
//     last_frame_time: Instant,
//     should_quit: bool,
//     picker: Picker, // Store the picker
// }
//
//
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Set up terminal
//     enable_raw_mode()?;
//     let mut stdout = io::stdout();
//     execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
//     let backend = CrosstermBackend::new(stdout);
//     let mut terminal = Terminal::new(backend)?;
//
//     // Create app and run it
//     let app = initialize_app()?;
//     let res = run_app(&mut terminal, app);
//
//     // Restore terminal
//     disable_raw_mode()?;
//     execute!(
//         terminal.backend_mut(),
//         LeaveAlternateScreen,
//         DisableMouseCapture
//     )?;
//     terminal.show_cursor()?;
//
//     if let Err(err) = res {
//         println!("{:?}", err)
//     }
//
//     Ok(())
// }
//
// fn initialize_app() -> Result<App, Box<dyn std::error::Error>> {
//     // Automatically detect terminal capabilities using Picker
//     let picker = Picker::from_query_stdio().unwrap_or_else(|_| {
//         Picker::from_fontsize((8, 12))
//     });
//
//     // Open the GIF file
//     let file = File::open("./assets/cat.gif")?;
//     let decoder = GifDecoder::new(BufReader::new(file))?; // Wrap file in BufReader
//
//     // Get the frame delay from the GIF
//     let mut frame_duration = Duration::from_millis(100); // Default fallback
//
//     // Extract all frames from the GIF
//     let frames = decoder.into_frames()
//         .collect_frames()?
//         .into_iter()
//         .map(|frame| {
//             // Get frame delay information
//             let delay = frame.delay().numer_denom_ms();
//             // Calculate the duration in milliseconds
//             frame_duration = Duration::from_millis(delay.0 as u64 / delay.1 as u64);
//
//             image::DynamicImage::ImageRgba8(frame.into_buffer())
//         })
//         .collect::<Vec<_>>();
//
//     // Need at least one frame
//     if frames.is_empty() {
//         return Err("No frames found in GIF".into());
//     }
//
//     // Initial frame
//     let image = picker.new_resize_protocol(frames[0].clone());
//
//     Ok(App {
//         image,
//         frames,
//         current_frame: 0,
//         frame_duration,
//         last_frame_time: Instant::now(),
//         should_quit: false,
//         picker, // Store the picker
//     })
// }
//
// fn run_app<B: ratatui::backend::Backend>(
//     terminal: &mut Terminal<B>,
//     mut app: App,
// ) -> io::Result<()> {
//     loop {
//         terminal.draw(|f| ui(f, &mut app))?;
//
//         // Check the image encoding result
//         if let Some(result) = app.image.last_encoding_result() {
//             if let Err(e) = result {
//                 eprintln!("Image encoding error: {:?}", e);
//             }
//         }
//
//         // Check if it's time to change frames
//         let now = Instant::now();
//         if now.duration_since(app.last_frame_time) >= app.frame_duration {
//             // Update to next frame
//             app.current_frame = (app.current_frame + 1) % app.frames.len();
//
//             // Get the picker again (or store it in App)
//             let picker = Picker::from_query_stdio().unwrap_or_else(|_| {
//                 Picker::from_fontsize((8, 12))
//             });
//
//             // Update the image with the new frame
//             app.image = picker.new_resize_protocol(app.frames[app.current_frame].clone());
//
//             app.last_frame_time = now;
//         }
//
//         // Poll for events with a shorter timeout to ensure smooth animations
//         if event::poll(Duration::from_millis(16))? {
//             if let Event::Key(key) = event::read()? {
//                 match key.code {
//                     KeyCode::Char('q') | KeyCode::Esc => {
//                         app.should_quit = true;
//                     }
//                     _ => {}
//                 }
//             }
//         }
//
//         if app.should_quit {
//             break;
//         }
//     }
//     Ok(())
// }
//
// fn ui(f: &mut Frame<'_>, app: &mut App) {
//     // Create a layout
//     let chunks = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([Constraint::Percentage(100)].as_ref())
//         .split(f.size());
//
//     // The image widget
//     let image = StatefulImage::default();
//
//     // Render with the protocol state
//     f.render_stateful_widget(image, chunks[0], &mut app.image);
// }
