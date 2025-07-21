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

    // start the app
    let mut app = App::new();
    app.run()?;

    // restore terminal view
    ratatui::restore();

    Ok(())
}



























