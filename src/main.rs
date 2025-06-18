mod app;
mod screens;
mod mal;
mod handlers;
mod utils;

use crate::app::App;
use anyhow::Result;
use mal::models::anime::fields;

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
        let anime = animes.get(0).unwrap();
        println!("{}: {}",
            anime.title,
            anime.start_season
        );

        // for anime in animes {
        //     println!("{}: {}",
        //         anime.get(fields::TITLE).unwrap_or("unknown"), 
        //         anime.get(fields::START_SEASON).unwrap_or("unknown")
        //     );
        //     println!();
        // }
    };
    Ok(())
}



























