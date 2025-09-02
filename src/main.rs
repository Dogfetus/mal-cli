mod app;
mod screens;
mod mal;
mod handlers;
mod utils;
mod config;
mod player;

use crate::app::App;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {

    let terminal = ratatui::init();

    // start the app
    let mut app = App::new(terminal);
    app.run()?;

    ratatui::restore();

    Ok(())
}



























