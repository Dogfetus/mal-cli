use ratatui::style::Color;

// Configuration for colors used in the application
pub const PRIMARY_COLOR: Color = Color::DarkGray; 
pub const SECONDARY_COLOR: Color = Color::White;
pub const HIGHLIGHT_COLOR: Color = Color::LightCyan;
pub const SECOND_HIGHLIGHT_COLOR: Color = Color::LightYellow;
pub const ERROR_COLOR: Color = Color::Red;
pub const TEXT_COLOR: Color = Color::White; 
pub const SECOND_TEXT_COLOR: Color = Color::White; 

// Anime List Colors
pub const WATCHING_COLOR: Color = Color::Rgb(64, 201, 255);
pub const COMPLETED_COLOR: Color = Color::Rgb(83, 209, 131);
pub const ON_HOLD_COLOR: Color = Color::Rgb(181, 105, 16);
pub const DROPPED_COLOR: Color = Color::Rgb(163, 0, 0);
pub const PLAN_TO_WATCH_COLOR: Color = Color::Rgb(176, 86, 255);



pub fn anime_list_colors(status: impl AsRef<str>) -> Color {
    match status.as_ref().to_lowercase().as_str() {
        "watching" | "rewatching" => WATCHING_COLOR,
        "completed" => COMPLETED_COLOR,
        "on hold" | "on-hold" => ON_HOLD_COLOR,
        "dropped" => DROPPED_COLOR,
        "plan to watch" => PLAN_TO_WATCH_COLOR,
        _ => PRIMARY_COLOR,
    }
}
