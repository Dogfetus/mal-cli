use std::{collections::HashMap, fmt};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Anime {
    #[serde(default)]
    pub id: u16,
    #[serde(default)]
    pub title: String,
    pub main_picture: Option<Pictures>,
    pub alternative_titles: Option<AlternativeTitles>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub synopsis: Option<String>,
    pub mean: Option<f32>,
    pub rank: Option<u16>,
    pub popularity: Option<u16>,
    pub num_list_users: Option<u16>,
    pub num_scoring_users: Option<u16>,
    pub nsfw: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub media_type: Option<String>,
    pub status: Option<String>,
    pub genres: Option<Vec<Genre>>,
    pub my_list_status: Option<MyListStatus>,
    pub num_episodes: Option<u16>,
    #[serde(default)]
    pub start_season: StartSeason,
    pub broadcast: Option<Broadcast>,
    pub source: Option<String>,
    pub average_episode_duration: Option<u16>,
    pub rating: Option<String>,
    pub pictures: Option<Vec<Pictures>>,
    pub background: Option<String>,
    pub related_anime: Option<Vec<RelatedAnime>>,
    pub related_manga: Option<Vec<RelatedManga>>,
    pub recommendations: Option<Vec<Recommendation>>,
    pub studios: Option<Vec<Studio>>,
    pub statistics: Option<Statistics>,

}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimeResponse {
    data: Vec<Anime>,
    paging: Option<Value>,
}

impl Anime {
    pub fn empty() -> Self {
        Self {
            id: 0,
            title: String::new(),
            main_picture: None,
            alternative_titles: None,
            start_date: None,
            end_date: None,
            synopsis: None,
            mean: None,
            rank: None,
            popularity: None,
            num_list_users: None,
            num_scoring_users: None,
            nsfw: None,
            created_at: None,
            updated_at: None,
            media_type: None,
            status: None,
            genres: None,
            my_list_status: None,
            num_episodes: None,
            start_season: StartSeason::default(),
            broadcast: None,
            source: None,
            average_episode_duration: None,
            rating: None,
            pictures: None,
            background: None,
            related_anime: None,
            related_manga: None,
            recommendations: None,
            studios: None,
            statistics: None
        }
    }

    pub fn from_body(body: &Value) -> Vec<Anime> {
        match serde_json::from_value::<AnimeResponse>(body.clone()) {
            Ok(response) => response.data,
            Err(e) => {
                eprintln!("Failed to parse anime response: {}", e);
                // Print the actual JSON for debugging
                eprintln!("JSON structure: {}", serde_json::to_string_pretty(body).unwrap_or_default());
                Vec::new()
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pictures {
    pub large: Option<String>,
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlternativeTitles {
    pub synonyms: Option<Vec<String>>,
    pub en: Option<String>,
    pub ja: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Genre{
    pub id: u16,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyListStatus {
    pub status: Option<String>,
    pub score: Option<u8>,
    pub num_episodes_watched: Option<u16>,
    pub is_rewatching: Option<bool>,
    pub start_date: Option<String>,
    pub finish_date: Option<String>,
    pub priority: Option<u8>,
    pub num_times_rewatched: Option<u8>,
    pub rewatch_value: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub comments: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StartSeason {
    pub year: Option<u16>,
    pub season: Option<String>,
}

impl Default for StartSeason {
    fn default() -> Self {
        StartSeason {
            year: None,
            season: None,
        }
    }
}

impl fmt::Display for StartSeason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.year, &self.season) {
            (Some(year), Some(season)) => write!(f, "{} {}", year, season),
            (Some(year), None) => write!(f, "{}", year),
            (None, Some(season)) => write!(f, "{}", season),
            (None, None) => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Broadcast {
    pub day_of_the_week: Option<String>,
    pub start_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Studio {
    pub id: u16,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelatedAnime {
    pub node: Node,
    pub relation_type: Option<String>,
    pub relation_type_formatted: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Recommendation {
    pub node: Node,
    pub num_recommendations: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Status {
    pub watching: u16,
    pub completed: u16,
    pub on_hold: u16,
    pub dropped: u16,
    pub plan_to_watch: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Statistics {
    pub status: Status,
    pub num_list_users: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelatedManga {
    // TODO: related manga when adding manga
    pub node: Node,
    pub relation_type: String,
    pub relation_type_formatted: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node {
    pub id: u16,
    pub title: String,
    pub main_picture: Option<Pictures>,
}
