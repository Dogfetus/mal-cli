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

    
    let mal = mal::MalClient::new();
    if let Some(animes) = mal.get_current_season(0, 50){


        for anime in animes.clone() {
            println!("{}: {}",
                anime.title, 
                anime.start_season
            );
            println!();
        }

        println!("Current season animes: {}", animes.len());
    };
    Ok(())
}



























