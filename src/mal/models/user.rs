use crate::{mal::{network::fetch_user, Fetchable}, utils::imageManager::HasDisplayableImage};

use serde::{Deserialize, Serialize};


fn default_picture() -> String {
    "https://dogfetus.no/image/pfp".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    #[serde(default)]
    pub id: usize,
    #[serde(default)]
    pub name: String,
    #[serde(default= "default_picture")]
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
    pub anime_statistics: AnimeStatistics,
    #[serde(default)]
    pub time_zone: String,
    #[serde(default)]
    pub is_supporter: bool,
}

impl User {
    pub fn empty() -> Self {
        Self {
            id: 0,
            name: String::new(),
            picture: String::new(),
            gender: String::new(),
            birthday: String::new(),
            location: String::new(),
            joined_at: String::new(),
            anime_statistics: AnimeStatistics::default(),
            time_zone: String::new(),
            is_supporter: false,
        }
    }
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
    pub num_days_watching: f64,
    #[serde(default)]
    pub num_days_completed: f64,
    #[serde(default)]
    pub num_days_on_hold: f64,
    #[serde(default)]
    pub num_days_dropped: f64,
    #[serde(default)]
    pub num_days: f64,
    #[serde(default)]
    pub num_episodes: usize,
    #[serde(default)]
    pub num_times_rewatched: usize,
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
            num_days_watching: 0.0,
            num_days_completed: 0.0,
            num_days_on_hold: 0.0,
            num_days_dropped: 0.0,
            num_days: 0.0,
            num_episodes: 0,
            num_times_rewatched: 0,
            mean_score: 0.0,
        }
    }
}

impl Fetchable for User {
    type Response = Self;
    type Output = Self;

    fn fetch(
        token: String,
        url: String,
        parameters: Vec<(String, String)>,
    ) -> Result<Self::Response, Box<dyn std::error::Error>> {
        fetch_user(token, url, parameters)
    }

    fn from_response(response: Self::Response) -> Self::Output {
        response
    }
}


impl HasDisplayableImage for User {
    fn get_displayable_image(&self) -> Option<(usize, String)> {
        if self.picture.is_empty() {
            return None;
        }
        Some((self.id, self.picture.clone()))
    }
}

