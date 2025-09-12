mod app;
mod config;
mod handlers;
mod mal;
mod player;
mod screens;
mod utils;

use crate::app::App;
use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

fn parse_cli() -> bool {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-v" | "--version" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return true;
            }
            "-e" | "--edit" => {
                config::open_in_editor();
                return true;
            }
            "-h" | "--help" => {
                println!("Usage: anime-tui [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -h, --help       Show this help message");
                println!("  -v, --version    Show version information");
                println!("  -e, --edit       Edit the configuration file");
                return true;
            }
            _ => {}
        }
    }

    false
}

#[tokio::main]
async fn main() -> Result<()> {

    let run_command = parse_cli();
    if run_command {
        return Ok(());
    }

    let terminal = ratatui::init();
    let config = config::read_from_file();

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
