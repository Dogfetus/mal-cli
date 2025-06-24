use std::{collections::HashMap, fmt::{self, Display}};
use serde::{Deserialize, Serialize};
use serde_json::Value;


// season limit (first season ever) : year: 1917 season: winter

fn na() -> String{
    "N/A".to_string()
}

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
    pub id: u64,
    #[serde(default="na")]
    pub title: String,
    #[serde(default)]
    pub main_picture: Pictures,
    #[serde(default)]
    pub alternative_titles: AlternativeTitles,
    #[serde(default="na")]
    pub start_date: String,
    #[serde(default="na")]
    pub end_date: String,
    #[serde(default="na")]
    pub synopsis: String,
    #[serde(default)]
    pub mean: f32,
    #[serde(default)]
    pub rank: u64,
    #[serde(default)]
    pub popularity: u64,
    #[serde(default)]
    pub num_list_users: u64,
    #[serde(default)]
    pub num_scoring_users: u64,
    #[serde(default="na")]
    pub nsfw: String,
    #[serde(default="na")]
    pub created_at: String,
    #[serde(default="na")]
    pub updated_at: String,
    #[serde(default="na")]
    pub media_type: String,
    #[serde(default="na")]
    pub status: String,
    #[serde(default)]
    pub genres: Vec<Genre>,
    pub my_list_status: Option<MyListStatus>,
    #[serde(default)]
    pub num_episodes: u64,
    #[serde(default)]
    pub start_season: StartSeason,
    pub broadcast: Option<Broadcast>,
    #[serde(default="na")]
    pub source: String,
    #[serde(default)]
    pub average_episode_duration: u64,
    #[serde(default="na")]
    pub rating: String,
    pub pictures: Option<Vec<Pictures>>,
    #[serde(default="na")]
    pub background: String,
    pub related_anime: Option<Vec<RelatedAnime>>,
    pub related_manga: Option<Vec<RelatedManga>>,
    pub recommendations: Option<Vec<Recommendation>>,
    #[serde(default)]
    pub studios: Vec<Studio>,
    pub statistics: Option<Statistics>,
}

impl Anime {
    pub fn empty() -> Self {
        Self {
            id: 0,
            title: String::new(),
            main_picture: Pictures::default(),
            alternative_titles: AlternativeTitles::default(),
            start_date: String::new(),
            end_date: String::new(),
            synopsis: String::new(),
            mean: 0.0,
            rank: 0,
            popularity: 0,
            num_list_users: 0,
            num_scoring_users: 0,
            nsfw: String::new(),
            created_at: String::new(),
            updated_at: String::new(),
            media_type: String::new(),
            status: String::new(),
            genres: Vec::new(),
            my_list_status: None,
            num_episodes: 0,
            start_season: StartSeason::default(),
            broadcast: None,
            source: String::new(),
            average_episode_duration: 0,
            rating: String::new(),
            pictures: None,
            background: String::new(),
            related_anime: None,
            related_manga: None,
            recommendations: None,
            studios: Vec::new(),
            statistics: None
        }
    }

    pub fn from_response(response: AnimeResponse) -> Vec<Self> {
        response.data.into_iter().map(|anime_node| anime_node.node).collect()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Page{
    #[serde(default="na")]
    pub previous: String,
    #[serde(default="na")]
    pub next: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimeResponse {
    pub data: Vec<AnimeNode>,
    pub paging: Option<Page>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimeNode {
    pub node: Anime,
    pub ranking: Option<u64>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ranking {
    rank: u16,
    previous_rank: Option<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pictures {
    #[serde(default="na")]
    pub large: String,
    #[serde(default="na")]
    pub medium: String,
}

impl Default for Pictures {
    fn default() -> Self {
        Pictures {
            large: String::new(),
            medium: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlternativeTitles {
    pub synonyms: Option<Vec<String>>,
    #[serde(default="na")]
    pub en: String,
    #[serde(default="na")]
    pub ja: String,
}

impl Default for AlternativeTitles {
    fn default() -> Self {
        AlternativeTitles {
            synonyms: None,
            en: String::new(), 
            ja: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Genre{
    pub id: u64,
    pub name: String,
}

impl fmt::Display for Genre {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyListStatus {
    #[serde(default="na")]
    pub status: String,
    pub score: Option<u8>,
    pub num_episodes_watched: Option<u32>,
    pub is_rewatching: Option<bool>,
    #[serde(default="na")]
    pub start_date: String,
    #[serde(default="na")]
    pub finish_date: String,
    pub priority: Option<u8>,
    pub num_times_rewatched: Option<u8>,
    pub rewatch_value: Option<u8>,
    pub tags: Option<Vec<String>>,
    #[serde(default="na")]
    pub comments: String,
    #[serde(default="na")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StartSeason {
    #[serde(default)]
    pub year: u16,
    #[serde(default="na")]
    pub season: String,
}

impl Default for StartSeason {
    fn default() -> Self {
        StartSeason {
            year: 0,
            season: String::new(),
        }
    }
}

impl fmt::Display for StartSeason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.year == 0 && !self.season.is_empty() {
            write!(f, "{}", self.season)
        }
        else if self.season.is_empty() && self.year != 0 {
            write!(f, "{}", self.year)
        }
        else if self.year == 0 && self.season.is_empty() {
            write!(f, "N/A")
        }
        else {
            write!(f, "{} {}", self.season, self.year)
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Broadcast {
    #[serde(default="na")]
    pub day_of_the_week: String,
    #[serde(default="na")]
    pub start_time: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Studio {
    pub id: u64,
    pub name: String,
}

impl fmt::Display for Studio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelatedAnime {
    pub node: Node,
    #[serde(default="na")]
    pub relation_type: String,
    #[serde(default="na")]
    pub relation_type_formatted: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Recommendation {
    pub node: Node,
    pub num_recommendations: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Status {
    pub watching: u64,
    pub completed: u64,
    pub on_hold: u64,
    pub dropped: u64,
    pub plan_to_watch: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Statistics {
    pub status: Status,
    pub num_list_users: u64,
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
    pub id: u64,
    pub title: String,
    pub main_picture: Option<Pictures>,
}
