use serde::Deserialize;

//
// 1) Search shows: { data: { shows: { edges: [ { _id, name, availableEpisodes } ] } } }
//
#[derive(Debug, Deserialize)]
pub struct ShowSearch {
    pub data: ShowData,
}

#[derive(Debug, Deserialize)]
pub struct ShowData {
    pub shows: Shows,
}

#[derive(Debug, Deserialize)]
pub struct Shows {
    pub edges: Vec<ShowEdge>,
}

#[derive(Debug, Deserialize)]
pub struct ShowEdge {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(rename = "availableEpisodes")]
    pub available_episodes: AvailableEpisodes,
}

#[derive(Debug, Deserialize)]
pub struct AvailableEpisodes {
    pub sub: u32,
    pub dub: u32,
    pub raw: u32,
}



//
// 2) Show info (episodes list): { data: { show: { _id, availableEpisodesDetail: { sub:[..], dub:[..], raw:[..] } } } }
//
#[derive(Debug, Deserialize)]
pub struct InfoSearch {
    pub data: InfoData,
}

#[derive(Debug, Deserialize)]
pub struct InfoData {
    pub show: SingleShow,
}

#[derive(Debug, Deserialize)]
pub struct SingleShow {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "availableEpisodesDetail")]
    pub available_episodes_detail: AvailableEpisodesDetail,
}

#[derive(Debug, Deserialize)]
pub struct AvailableEpisodesDetail {
    pub sub: Vec<String>,
    pub dub: Vec<String>,
    pub raw: Vec<String>,
}



//
// 3) Episode sources: { data: { episode: { episodeString, sourceUrls:[...] } } }
//
#[derive(Debug, Deserialize)]
pub struct EpisodeSearch {
    pub data: EpisodeDataRoot,
}

#[derive(Debug, Deserialize)]
pub struct EpisodeDataRoot {
    pub episode: Episode,
}

#[derive(Debug, Deserialize)]
pub struct Episode {
    #[serde(rename = "episodeString")]
    pub episode_string: String,
    #[serde(rename = "sourceUrls")]
    pub source_urls: Vec<SourceUrl>,
}

#[derive(Debug, Deserialize)]
pub struct SourceUrl {
    #[serde(rename = "sourceUrl")]
    pub source_url: String,
    pub priority: Option<f64>,
    #[serde(rename = "sourceName")]
    pub source_name: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,       // "player" / "iframe"
    #[serde(rename = "className")]
    pub class_name: Option<String>,
    #[serde(rename = "streamerId")]
    pub streamer_id: Option<String>,
    pub sandbox: Option<String>,
    pub downloads: Option<Downloads>,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    #[serde(rename = "sourceName")]
    pub source_name: Option<String>,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
}

