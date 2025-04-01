mod app;
mod ui;
mod mal;
mod models;

use std::io;

use crate::{
    app::{App, CurrentScreen, CurrentlyEditing},
    ui::ui,
};


fn main() -> io::Result<()> {

    let terminal = ratatui::init();
    let mut app = App::new();

    results = app.run(&terminal);

    ratatui::restore();
    reults

}
