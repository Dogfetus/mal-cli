use ratatui::style::Color;

// Configuration for colors used in the application
pub const PRIMARY_COLOR: Color = Color::DarkGray; 
pub const SECONDARY_COLOR: Color = Color::White;
pub const HIGHLIGHT_COLOR: Color = Color::Cyan;
pub const ERROR_COLOR: Color = Color::Red;

// Anime List Colors
pub const WATCHING_COLOR: Color = Color::Rgb(64, 201, 255);
pub const COMPLETED_COLOR: Color = Color::Rgb(83, 209, 131);
pub const ON_HOLD_COLOR: Color = Color::Rgb(181, 105, 16);
pub const DROPPED_COLOR: Color = Color::Rgb(163, 0, 0);
pub const PLAN_TO_WATCH_COLOR: Color = Color::Rgb(176, 86, 255);



pub fn anime_list_colors(status: &String) -> Color {
    match status.as_str() {
        "watching" | "rewatching" => WATCHING_COLOR,
        "completed" => COMPLETED_COLOR,
        "on hold" => ON_HOLD_COLOR,
        "dropped" => DROPPED_COLOR,
        "plan to watch" => PLAN_TO_WATCH_COLOR,
        _ => Color::Gray,
    }
}
