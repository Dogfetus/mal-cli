use std::path::PathBuf;
use std::process::Command;
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

pub struct Config {
    pub api_endpoint: String,
    pub items_per_page: usize,
    pub enable_notifications: bool,
    pub notification_command: String,
    pub default_video_quality: String,
    pub use_dark_mode: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            api_endpoint: "https://api.example.com".to_string(),
            items_per_page: 20,
            enable_notifications: true,
            notification_command: "notify-send".to_string(),
            default_video_quality: "1080p".to_string(),
            use_dark_mode: true,
        }
    }
}

pub fn open_in_editor() {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or("nano".to_string());

    let config_path = get_config_path().join("config.toml");

    Command::new(editor)
        .arg(config_path)
        .status()
        .map_err(|e| {
            eprintln!("Failed to open editor: {}", e);
        })
        .ok();
}

pub fn get_app_dir() -> PathBuf {
    std::env::var("HOME").ok()
    .map(|home| PathBuf::from(home)
    .join(".local/share/mal-cli"))
    .expect("Failed to get app directory")
} 

pub fn get_config_path() -> PathBuf {
    std::env::var("HOME").ok()
    .map(|home| PathBuf::from(home)
    .join(".config/mal-cli"))
    .expect("Failed to get app directory")
}

pub fn read_from_file() -> Config {
    let config_path = get_config_path().join("config.toml");
    if !config_path.exists() {
        return Config::new();
    }

    let content = std::fs::read_to_string(&config_path).unwrap_or_else(|_| {
        eprintln!("Failed to read config file, using default configuration.");
        String::new()
    });

    Config::new() // Placeholder
}

