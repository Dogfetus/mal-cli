#[allow(unused)]
pub mod fields {
    pub const ID: &str = "id";
    pub const TITLE: &str = "title";
    pub const MAIN_PICTURE: &str = "main_picture";
    pub const ALTERNATIVE_TITLES: &str = "alternative_titles";
    pub const START_DATE: &str = "start_date";
    pub const END_DATE: &str = "end_date";
    pub const SYNOPSIS: &str = "synopsis";
    pub const MEAN: &str = "mean";
    pub const RANK: &str = "rank";
    pub const POPULARITY: &str = "popularity";
    pub const NUM_LIST_USERS: &str = "num_list_users";
    pub const NUM_EPISODES: &str = "num_episodes";
    pub const STATUS: &str = "status";
    pub const PICTURES: &str = "pictures";
    pub const BACKGROUND: &str = "background";
    pub const RELATED_ANIME: &str = "related_anime";
    pub const RELATED_MANGA: &str = "related_manga";
    pub const GENRES: &str = "genres";
    pub const STUDIOS: &str = "studios";
    pub const RECOMMENDATIONS: &str = "recommendations";
    pub const STATISTICS: &str = "statistics";
    pub const MY_LIST_STATUS: &str = "my_list_status";
    pub const BROADCAST: &str = "broadcast";
    pub const OPENING_THEMES: &str = "opening_themes";
    pub const ENDING_THEMES: &str = "ending_themes";
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Anime {
    pub id: i32,
    pub title: String,
    pub large_picture: String,
    pub medium_picture: String,
    pub alternative_titles: Vec<String>,
    pub start_date: String,
    pub end_date: String,
    pub synopsis: String,
    pub mean: f32,
    pub rank: i32,
    pub popularity: i32,
    pub num_list_users: i32,
    pub num_episodes: i32,
    pub status: String,
    pub pictures: Vec<String>,
    pub background: String,
    pub related_anime: Vec<String>,
    pub related_manga: Vec<String>,
    pub genres: Vec<String>,
    pub studios: Vec<String>,
    pub recommendations: Vec<String>,
    pub statistics: Vec<String>,
    pub my_list_status: String,
    pub broadcast: String,
    pub opening_themes: Vec<String>,
    pub ending_themes: Vec<String>,
}

impl Anime {
    pub fn new(json: &serde_json::Value) -> Self {
        Self {
            id: json["id"].as_i64().unwrap_or(0) as i32,
            title: json["title"].as_str().unwrap_or("").to_string(),
            large_picture: json["main_picture"]["large"].as_str().unwrap_or("").to_string(),
            medium_picture: json["main_picture"]["medium"].as_str().unwrap_or("").to_string(),
            alternative_titles: Self::extract_alt_titles(json),
            start_date: json["start_date"].as_str().unwrap_or("").to_string(),
            end_date: json["end_date"].as_str().unwrap_or("").to_string(),
            synopsis: json["synopsis"].as_str().unwrap_or("").to_string(),
            mean: json["mean"].as_f64().unwrap_or(0.0) as f32,
            rank: json["rank"].as_i64().unwrap_or(0) as i32,
            popularity: json["popularity"].as_i64().unwrap_or(0) as i32,
            num_list_users: json["num_list_users"].as_i64().unwrap_or(0) as i32,
            num_episodes: json["num_episodes"].as_i64().unwrap_or(0) as i32,
            status: json["status"].as_str().unwrap_or("").to_string(),
            pictures: Self::extract_pictures(json),
            background: json["background"].as_str().unwrap_or("").to_string(),
            related_anime: Self::extract_related_anime(json),
            related_manga: Self::extract_related_manga(json),
            genres: Self::extract_genres(json),
            studios: Self::extract_studios(json),
            recommendations: Self::extract_recommendations(json),
            statistics: Self::extract_statistics(json),
            my_list_status: json["my_list_status"]["status"].as_str().unwrap_or("").to_string(),
            broadcast: json["broadcast"].as_str().unwrap_or("").to_string(),
            opening_themes: Self::extract_opening_themes(json),
            ending_themes: Self::extract_ending_themes(json),
        }
    }
    fn extract_genres(json: &serde_json::Value) -> Vec<String> {
        json["genres"].as_array()
            .map(|genres| genres.iter()
                .filter_map(|g| g["name"].as_str().map(String::from))
                .collect())
            .unwrap_or_default()
    }
    fn extract_studios(json: &serde_json::Value) -> Vec<String> {
        json["studios"].as_array()
            .map(|studios| studios.iter()
                .filter_map(|s| s["name"].as_str().map(String::from))
                .collect())
            .unwrap_or_default()
    }
    fn extract_pictures(json: &serde_json::Value) -> Vec<String> {
        json["pictures"].as_array()
            .map(|pics| pics.iter()
                .filter_map(|p| p["large"].as_str().or_else(|| p["medium"].as_str()))
                .map(String::from)
                .collect())
            .unwrap_or_default()
    }
    fn extract_alt_titles(json: &serde_json::Value) -> Vec<String> {
        let mut titles = Vec::new();
        if let Some(alt) = json["alternative_titles"].as_object() {
            if let Some(en) = alt["en"].as_str() {
                titles.push(en.to_string());
            }
            if let Some(ja) = alt["ja"].as_str() {
                titles.push(ja.to_string());
            }
        }
        titles
    }
    fn extract_related_anime(json: &serde_json::Value) -> Vec<String> {
        json["related_anime"].as_array()
            .map(|related| related.iter()
                .filter_map(|r| r["node"]["title"].as_str().map(String::from))
                .collect())
            .unwrap_or_default()
    }
    fn extract_related_manga(json: &serde_json::Value) -> Vec<String> {
        json["related_manga"].as_array()
            .map(|related| related.iter()
                .filter_map(|r| r["node"]["title"].as_str().map(String::from))
                .collect())
            .unwrap_or_default()
    }
    fn extract_recommendations(json: &serde_json::Value) -> Vec<String> {
        json["recommendations"].as_array()
            .map(|recs| recs.iter()
                .filter_map(|r| r["node"]["title"].as_str().map(String::from))
                .collect())
            .unwrap_or_default()
    }
    fn extract_statistics(json: &serde_json::Value) -> Vec<String> {
        // Convert statistics object to strings - adjust as needed
        let mut stats = Vec::new();
        if let Some(stat_obj) = json["statistics"].as_object() {
            for (key, value) in stat_obj {
                stats.push(format!("{}: {}", key, value));
            }
        }
        stats
    }
    fn extract_opening_themes(json: &serde_json::Value) -> Vec<String> {
        json["opening_themes"].as_array()
            .map(|themes| themes.iter()
                .filter_map(|t| t["text"].as_str().map(String::from))
                .collect())
            .unwrap_or_default()
    }
    fn extract_ending_themes(json: &serde_json::Value) -> Vec<String> {
        json["ending_themes"].as_array()
            .map(|themes| themes.iter()
                .filter_map(|t| t["text"].as_str().map(String::from))
                .collect())
            .unwrap_or_default()
    }

    pub fn empty() -> Self {
        Self {
            id: 0,
            title: String::new(),
            large_picture: String::new(),
            medium_picture: String::new(),
            alternative_titles: Vec::new(),
            start_date: String::new(),
            end_date: String::new(),
            synopsis: String::new(),
            mean: 0.0,
            rank: 0,
            popularity: 0,
            num_list_users: 0,
            num_episodes: 0,
            status: String::new(),
            pictures: Vec::new(),
            background: String::new(),
            related_anime: Vec::new(),
            related_manga: Vec::new(),
            genres: Vec::new(),
            studios: Vec::new(),
            recommendations: Vec::new(),
            statistics: Vec::new(),
            my_list_status: String::new(),
            broadcast: String::new(),
            opening_themes: Vec::new(),
            ending_themes: Vec::new(),
        }
    }
}

// {
//   "data": [
//     {
//       "node": {
//         "id": 57859,
//         "title": "Egao no Taenai Shokuba desu.",
//         "main_picture": {
//           "medium": "https://cdn.myanimelist.net/images/anime/1826/148165.jpg",
//           "large": "https://cdn.myanimelist.net/images/anime/1826/148165l.jpg"
//         },
//         "alternative_titles": {
//           "synonyms": [
//             "A Workplace Where You Can't Help But Smile"
//           ],
//           "en": "A Mangaka's Weirdly Wonderful Workplace",
//           "ja": "笑顔のたえない職場です。"
//         },
//         "start_date": "2025-10",
//         "synopsis": "New shoujo manga artist Nana Futami works hard every day while being supported by Kaede Satou, her female editor who is older than her, and Mizuki Hazama, her assistant. According to the girl herself, she sometimes drums up intense daydream delusions of occupational illness! A working girls comedy set in the entertainment industry, brought to you by an author who always draws various girls.\n\n(Source: Kodansha, translated)",
//         "popularity": 9747,
//         "num_list_users": 3989,
//         "num_episodes": 0,
//         "status": "not_yet_aired",
//         "genres": [
//           {
//             "id": 50,
//             "name": "Adult Cast"
//           },
//           {
//             "id": 52,
//             "name": "CGDCT"
//           },
//           {
//             "id": 4,
//             "name": "Comedy"
//           },
//           {
//             "id": 69,
//             "name": "Otaku Culture"
//           },
//           {
//             "id": 48,
//             "name": "Workplace"
//           }
//         ],
//         "studios": [
//           {
//             "id": 2698,
//             "name": "Voil"
//           }
//         ]
//       }
//     }
//   ],
//   "paging": {
//     "next": "https://api.myanimelist.net/v2/anime/season/2025/fall?offset=1&fields=id%2Ctitle%2Cmain_picture%2Calternative_titles%2Cstart_date%2Cend_date%2Csynopsis%2Cmean%2Crank%2Cpopularity%2Cnum_list_users%2Cnum_episodes%2Cstatus%2Cpictures%2Cbackground%2Crelated_anime%2Crelated_manga%2Cgenres%2Cstudios%2Crecommendations%2Cstatistics%2Cmy_list_status%2Cbroadcast%2Copening_themes%2Cending_themes&limit=1"
//   },
//   "season": {
//     "year": 2025,
//     "season": "fall"
//   }
// }
