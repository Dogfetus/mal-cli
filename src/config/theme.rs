use serde::{Deserialize, Serialize};
use ratatui::style::Color;

// Default color functions
fn def_primary() -> Color { Color::DarkGray }
fn def_secondary() -> Color { Color::White }
fn def_highlight() -> Color { Color::LightCyan }
fn def_second_highlight() -> Color { Color::LightYellow }
fn def_error() -> Color { Color::Red }
fn def_text() -> Color { Color::White }
fn def_second_text() -> Color { Color::White }
fn def_watching() -> Color { Color::Rgb(64, 201, 255) }
fn def_completed() -> Color { Color::Rgb(83, 209, 131) }
fn def_on_hold() -> Color { Color::Rgb(181, 105, 16) }
fn def_dropped() -> Color { Color::Rgb(163, 0, 0) }
fn def_plan_to_watch() -> Color { Color::Rgb(176, 86, 255) }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Theme {
    #[serde(default = "def_primary")]
    pub primary: Color,
    #[serde(default = "def_secondary")]
    pub secondary: Color,
    #[serde(default = "def_highlight")]
    pub highlight: Color,
    #[serde(default = "def_second_highlight")]
    pub second_highlight: Color,
    #[serde(default = "def_error")]
    pub error: Color,
    #[serde(default = "def_text")]
    pub text: Color,
    #[serde(default = "def_second_text")]
    pub second_text: Color,

    // color of anime list status
    #[serde(default = "def_watching")]
    pub watching: Color,
    #[serde(default = "def_completed")]
    pub completed: Color,
    #[serde(default = "def_on_hold")]
    pub on_hold: Color,
    #[serde(default = "def_dropped")]
    pub dropped: Color,
    #[serde(default = "def_plan_to_watch")]
    pub plan_to_watch: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: def_primary(),
            secondary: def_secondary(),
            highlight: def_highlight(),
            second_highlight: def_second_highlight(),
            error: def_error(),
            text: def_text(),
            second_text: def_second_text(),
            watching: def_watching(),
            completed: def_completed(),
            on_hold: def_on_hold(),
            dropped: def_dropped(),
            plan_to_watch: def_plan_to_watch(),
        }
    }
}

impl Theme {
    /// Get color for anime list status
    pub fn status_color(&self, status: impl AsRef<str>) -> Color {
        match status.as_ref().to_lowercase().as_str() {
            "watching" | "rewatching" => self.watching,
            "completed" => self.completed,
            "on hold" | "on-hold" => self.on_hold,
            "dropped" => self.dropped,
            "plan to watch" => self.plan_to_watch,
            _ => self.primary,
        }
    }
}
