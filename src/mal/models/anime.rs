use std::collections::HashMap;

use serde_json::Value;



#[allow(unused)]
pub mod fields {
    pub const ID: &str                       = "id";
    pub const TITLE: &str                    = "title";
    pub const MAIN_PICTURE: &str             = "main_picture";
    pub const ALTERNATIVE_TITLES: &str       = "alternative_titles";
    pub const START_DATE: &str               = "start_date";
    pub const END_DATE: &str                 = "end_date";
    pub const SYNOPSIS: &str                 = "synopsis";
    pub const MEAN: &str                     = "mean";
    pub const RANK: &str                     = "rank";
    pub const POPULARITY: &str               = "popularity";
    pub const NUM_LIST_USERS: &str           = "num_list_users";
    pub const NUM_SCORING_USERS: &str        = "num_scoring_users";
    pub const NSFW: &str                     = "nsfw";
    pub const CREATED_AT: &str               = "created_at";
    pub const UPDATED_AT: &str               = "updated_at";
    pub const MEDIA_TYPE: &str               = "media_type";
    pub const STATUS: &str                   = "status";
    pub const GENRES: &str                   = "genres";
    pub const MY_LIST_STATUS: &str           = "my_list_status";
    pub const NUM_EPISODES: &str             = "num_episodes";
    pub const START_SEASON: &str             = "start_season";
    pub const BROADCAST: &str                = "broadcast";
    pub const SOURCE: &str                   = "source";
    pub const AVERAGE_EPISODE_DURATION: &str = "average_episode_duration";
    pub const RATING: &str                   = "rating";
    pub const PICTURES: &str                 = "pictures";
    pub const BACKGROUND: &str               = "background";
    pub const RELATED_ANIME: &str            = "related_anime";
    pub const RELATED_MANGA: &str            = "related_manga";
    pub const RECOMMENDATIONS: &str          = "recommendations";
    pub const STUDIOS: &str                  = "studios";
    pub const STATISTICS: &str               = "statistics";
    pub const ALL: [&str; 32] = [
        ID, TITLE, MAIN_PICTURE, ALTERNATIVE_TITLES, START_DATE, END_DATE, SYNOPSIS, MEAN, RANK, POPULARITY, 
        NUM_LIST_USERS, NUM_SCORING_USERS, NSFW, CREATED_AT, UPDATED_AT, MEDIA_TYPE, STATUS, GENRES, 
        MY_LIST_STATUS, NUM_EPISODES, START_SEASON, BROADCAST, SOURCE, AVERAGE_EPISODE_DURATION, RATING, 
        PICTURES, BACKGROUND, RELATED_ANIME, RELATED_MANGA, RECOMMENDATIONS, STUDIOS, STATISTICS,
    ]; 
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Anime {
    fields: HashMap<String, String>,
}

impl Anime {
    pub fn from_body(body: &Value) -> Vec<Self> {
        let list = body["data"].as_array()
            .map(|arr| arr.clone())
            .unwrap_or_else(|| vec![body.clone()]);
        
        let mut anime_list = Vec::new();
        
        for item in list {
            let anime_data = item.get("node").unwrap_or(&item);
            anime_list.push(Self::from_node(anime_data));
        }
        
        anime_list
    }

    pub fn from_node(node: &Value) -> Self {
        let mut fields = HashMap::new();
        
        if let Some(obj) = node.as_object() {
            for (key, val) in obj {
                let string_value = Self::convert_json_value_to_string(val, key);
                
                if !string_value.is_empty() {
                    fields.insert(key.clone(), string_value);
                }
            }
        }
        
        Self { fields }
    }

    fn convert_json_value_to_string(val: &Value, key: &str) -> String {
        match val {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Array(arr) => {
                // Handle specific array fields
                match key {
                    "genres" | "studios" => {
                        arr.iter()
                            .filter_map(|item| item.get("name").and_then(|name| name.as_str()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    },
                    _ => serde_json::to_string(arr).unwrap_or_default()
                }
            },
            Value::Object(obj) => {
                // Handle specific object fields
                match key {
                    "main_picture" => {
                        obj.get("large")
                            .or_else(|| obj.get("medium"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    },
                    "alternative_titles" => {
                        obj.get("en")
                            .or_else(|| obj.get("ja"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    },
                    _ => serde_json::to_string(obj).unwrap_or_default()
                }
            },
            Value::Null => String::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        if let Some(value) = self.fields.get(key) {
            if !value.is_empty() {
                return Some(value);
            }
        }
        None
    }

    pub fn gets(&self, keys: &Vec<String>) -> Option<Vec<&String>> {
        let mut result = Vec::new();
        for key in keys {
            if let Some(value) = self.fields.get(key) {
                if value.is_empty() {
                    return None;
                }
                result.push(value);
            } else {
                return None;
            }
        }
        Some(result)
    }
}
