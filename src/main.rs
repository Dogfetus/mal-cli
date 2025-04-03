mod app;
mod ui;
mod mal;
mod models;
mod controller;


use std::io;
use crate::app::App;


fn main() -> io::Result<()> {

    let mut terminal = ratatui::init();
    let mut app = App::new();

    let results = app.run(&mut terminal);

    ratatui::restore();
    results

}

