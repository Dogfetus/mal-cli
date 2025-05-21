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
    pub title: String,
    pub mean: f32,
    pub status: String,
    pub synopsis: String,
}

impl Anime {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            mean: 0.0,
            status: String::new(),
            synopsis: String::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            title: String::new(),
            mean: 0.0,
            status: String::new(),
            synopsis: String::new(),
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
