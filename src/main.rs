mod app;
mod screens;
mod mal;
mod handlers;
mod utils;

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

    let mal_client = mal::MalClient::new();
    mal_client.get_current_season(0, 10);


    Ok(())
}



























