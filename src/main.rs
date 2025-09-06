mod app;
mod screens;
mod mal;
mod handlers;
mod utils;
mod config;
mod player;

use crate::app::App;
use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

fn parse_cli() -> bool {

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-v" | "--version" => return true,
            _ => {}
        }
    }

    false
}

#[tokio::main]
async fn main() -> Result<()> {
    let show_version = parse_cli(); 
    if show_version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let terminal = ratatui::init();

    // enable mouse capture
    crossterm::execute!(std::io::stderr(), EnableMouseCapture)?;

    // start the app
    let mut app = App::new(terminal);
    app.run()?;

    ratatui::restore();

    // disable mouse capture
    crossterm::execute!(std::io::stderr(), DisableMouseCapture)?;

    Ok(())
}

