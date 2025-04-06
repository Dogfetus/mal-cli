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


    let mut terminal = ratatui::init();
    let mut app = App::new();
    app.run(&mut terminal)?;
    ratatui::restore();

    mal::oauth_login().await;

    Ok(())

}




