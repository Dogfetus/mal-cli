use ratatui::style::Color;


pub fn anime_list_colors(list_type: &String) -> Color {
    match list_type.as_str() {
        "watching" | "rewatching" => Color::Rgb(64, 201, 255),
        "completed" => Color::Rgb(83, 209, 131),
        "on hold" => Color::Rgb(181, 105, 16),
        "dropped" => Color::Rgb(163, 0, 0),
        "plan to watch" => Color::Rgb(176, 86, 255),
        _ => Color::Gray,
    }

}
