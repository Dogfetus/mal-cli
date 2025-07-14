mod app;
mod screens;
mod mal;
mod handlers;
mod utils;
mod config;

use crate::app::App;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {

    // start the terminal view
    let mut terminal = ratatui::init();
    let mut app = App::new();
    app.run(&mut terminal)?;

    // restore
    ratatui::restore();

    Ok(())
}



























