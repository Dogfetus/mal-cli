use super::na;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    #[serde(default)]
    pub id: usize,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub picture: String,
    #[serde(default)]
    pub gender: String,
    #[serde(default)]
    pub birthday: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub joined_at: String,
    #[serde(default)]
    pub anime_atatistics: AnimeStatistics,
    #[serde(default)]
    pub time_zone: String,
    #[serde(default)]
    pub is_supporter: bool,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimeStatistics {
    #[serde(default)]
    pub num_items_watching: usize,
    #[serde(default)]
    pub num_items_completed: usize,
    #[serde(default)]
    pub num_items_on_hold: usize,
    #[serde(default)]
    pub num_items_dropped: usize,
    #[serde(default)]
    pub num_items_plan_to_watch: usize,
    #[serde(default)]
    pub num_items: usize,
    #[serde(default)]
    pub num_days_watched: f64,
    #[serde(default)]
    pub num_days_watching: String,
    #[serde(default)]
    pub num_days_completed: String,
    #[serde(default)]
    pub num_days_on_hold: String,
    #[serde(default)]
    pub num_days_dropped: String,
    #[serde(default)]
    pub num_episodes: usize,
    #[serde(default)]
    pub num_hours_watching: String,
    #[serde(default)]
    pub mean_score: f64,
}

impl Default for AnimeStatistics {
    fn default() -> Self {
        Self {
            num_items_watching: 0,
            num_items_completed: 0,
            num_items_on_hold: 0,
            num_items_dropped: 0,
            num_items_plan_to_watch: 0,
            num_items: 0,
            num_days_watched: 0.0,
            num_days_watching: na(),
            num_days_completed: na(),
            num_days_on_hold: na(),
            num_days_dropped: na(),
            num_episodes: 0,
            num_hours_watching: na(),
            mean_score: 0.0,
        }
    }
}
