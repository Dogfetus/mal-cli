mod navigation;

use crossterm::event::KeyCode;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

use navigation::Navigation;


const CONFIG_FILE: &str = "config.toml";

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


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub navigation: Navigation,
}

impl Config {
    pub fn default() -> Self {
        Self {
            navigation: Navigation {
                nav_up: vec![KeyCode::Up, KeyCode::Char('k')],
                nav_down: vec![KeyCode::Down, KeyCode::Char('j')],
                nav_left: vec![KeyCode::Left, KeyCode::Char('h')],
                nav_right: vec![KeyCode::Right, KeyCode::Char('l')],
            },
        }
    }


    // where configs are stored (default)
    pub fn config_dir() -> PathBuf {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".config/mal-cli"))
            .expect("Failed to get app directory")
    }


    // where logging and such is saved (episodes watched)
    pub fn data_dir() -> PathBuf {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".local/share/mal-cli"))
            .expect("Failed to get app directory")
    }


    // used to update the config file with new configs
    pub fn save_to_file(config: &Config) {
        let config_path = Self::config_dir();
        if !config_path.exists() {
            std::fs::create_dir_all(&config_path).unwrap_or_else(|_| {
                eprintln!("Failed to create config directory");
            });
        }
        let config_file_path = config_path.join(CONFIG_FILE);
        let toml = toml::to_string(&config)
            .map_err(|e| {
                eprintln!("Failed to serialize config: {}", e);
            })
            .unwrap_or_default();

        std::fs::write(&config_file_path, toml).unwrap_or_else(|_| {
            eprintln!("Failed to write config file");
        });
    }


    // creates default configs if no file exists already
    fn create_if_not_exists() {
        let config_path = Self::config_dir().join(CONFIG_FILE);
        if !config_path.exists() {
            Self::save_to_file(&Config::default());
        }
    }


    // edit the configs
    pub fn open_in_editor() {
        Self::create_if_not_exists();

        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or("nano".to_string());

        let config_path = Self::config_dir().join(CONFIG_FILE);

        Command::new(editor)
            .arg(config_path)
            .status()
            .map_err(|e| {
                eprintln!(
                    "Failed to open editor: {} try edit manually: ~/.config/mal-cli/config.toml",
                    e
                );
            })
            .ok();
    }


    // read the configs
    pub fn read_from_file() -> Config {
        Self::create_if_not_exists();

        // in case file generation fails use default
        let config_path = Self::config_dir().join(CONFIG_FILE);
        if !config_path.exists() {
            return Config::default();
        }

        println!("Loading config from: {:?}", config_path);

        let contents = std::fs::read_to_string(&config_path).unwrap_or_else(|_| {
            eprintln!("Failed to read config file, using default");
            String::new()
        });

        let config: Config = toml::from_str(&contents).unwrap_or_else(|_| {
            eprintln!("Failed to parse config file, using default");
            Config::default()
        });

        config
    }
}
