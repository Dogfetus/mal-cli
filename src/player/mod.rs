mod allanime;
use allanime::EpisodeSearch;
use allanime::LinksSearch;
use allanime::ShowEdge;
use allanime::ShowSearch;
use allanime::SourceUrl;
use regex::Regex;
use url::Url;

use crate::config::Config;
use crate::mal::models::anime::Anime;
use crate::mal::network::send_request;
use crate::mal::network::send_request_expect_text;
use crate::params;
use serde_json::json;
use std::io::ErrorKind;
use std::process::Command;

const BASE: &str = "https://allanime.day";
const API: &str = "https://api.allanime.day/api";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0";
const REF: &str = "https://allmanga.to";

#[derive(Debug, Clone)]
pub enum PlayError {
    NotReleased(Box<Anime>),
    CommandFailed {
        stderr: String,
        exit_code: i32,
        stdout: String,
    },
    NotFound(String),
    NoResults(String),
    Other(String),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PlayResult {
    pub episode: u32,
    pub current_time: String,
    pub total_time: String,
    pub percentage: u8,
    pub fully_watched: bool,
    pub completed: bool,
}

pub struct AnimePlayer {
    ansi_regex: Regex,

    //mpv regex:
    av_regex: Regex,
    exit_regex: Regex,

    // url Regex:
    wixmp_regex: Regex,
}

impl std::fmt::Display for PlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayError::NotReleased(anime) => write!(
                f,
                "\"{}\"\nis not yet released.",
                if anime.alternative_titles.en.is_empty() {
                    anime.title.clone()
                } else {
                    anime.alternative_titles.en.clone()
                }
            ),
            PlayError::CommandFailed {
                stderr,
                exit_code,
                stdout,
            } => {
                write!(
                    f,
                    "ani-cli replied:\nError: {}\nExit code: {}\nOutput: {}",
                    stderr, exit_code, stdout
                )
            }
            PlayError::NotFound(msg) => write!(f, "Can't seem to find ani-cli: \n{}", msg),
            PlayError::NoResults(msg) => write!(
                f,
                "ani-cli replied:\nError: {}\nthe anime might not be available yet",
                msg
            ),
            PlayError::Other(msg) => write!(f, "Error running ani-cli: \n{}", msg),
        }
    }
}

impl AnimePlayer {
    pub fn new() -> Self {
        AnimePlayer {
            ansi_regex: Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\([AB]|\r|\x1b[78]").unwrap(),
            av_regex: Regex::new(r"AV: (\d{2}:\d{2}:\d{2}) / (\d{2}:\d{2}:\d{2}) \((\d+)%\)")
                .unwrap(),
            exit_regex: Regex::new(r"Exiting\.\.\. \((.*?)\)").unwrap(),
            wixmp_regex: Regex::new(
                r#"^video\.wixstatic\.com/video/([^/]+)/,([^/]+),/mp4/file\.mp4$"#,
            )
            .unwrap(),
        }
    }

    pub fn extract_play_info(&self, stdout: &str, episode: u32) -> Option<PlayResult> {
        // return default if no output
        if stdout.is_empty() {
            return Some(PlayResult {
                current_time: "00:00:00".to_string(),
                total_time: "00:00:00".to_string(),
                completed: false,
                fully_watched: false,
                percentage: 0,
                episode,
            })
        }

        let last_av = if let Some(last_av) = stdout.rfind("AV: ") {
            let last_stdout = &stdout[last_av..];
            self.av_regex.captures(last_stdout)?
        } else {
            return None;
        };

        let exit_reason = self
            .exit_regex
            .captures(stdout)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str());

        let percentage = last_av[3].parse().unwrap_or(0);

        Some(PlayResult {
            current_time: last_av[1].to_string(),
            total_time: last_av[2].to_string(),
            completed: percentage >= 90,
            fully_watched: exit_reason == Some("End of file"),
            percentage,
            episode,
        })
    }

    pub fn play_episode_manually(
        &self,
        anime: &Anime,
        episode: u32,
    ) -> Result<PlayResult, PlayError> {
        if anime.status == "upcoming" {
            return Err(PlayError::NotReleased(Box::new(anime.clone())));
        }

        ratatui::restore();

        // hook
        if let Some(hook) = Config::global().player.pre_playback_hook.clone() {
            if let Err(e) = self.run_command(&hook, anime, episode, None) {
                eprintln!("Failed to run pre-playback hook: {}", e);
            }
        };


        // get available shows for the given anime title
        let shows = self.get_shows(anime.title.clone())?;

        // extract the correct show id from the list of shows
        let id = self.extract_correct_id(&shows, anime)?;

        // get the available episodes for the show
        let available_episodes = self.get_episode_providers(&id, episode)?;

        // extract the correct (the one with highest priority) episode from the list of available episodes
        let candidate = self.extract_best_candidate(&available_episodes)?;

        let result = if Config::global().player.disable_default_player {
            String::new()
        } else {
            self.play_video_in_mpv(&candidate)?
        };


        // hook
        if let Some(hook) = Config::global().player.post_playback_hook.clone() {
            if let Err(e) = self.run_command(&hook, anime, episode, Some(&candidate)) {
                eprintln!("Failed to run pre-playback hook: {}", e);
            }
        };

        // mark as completed
        if Config::global().player.allways_complete_episode {
            return Ok(PlayResult {
                current_time: "00:00:00".to_string(),
                total_time: "00:00:00".to_string(),
                completed: true,
                fully_watched: true,
                percentage: 100,
                episode,
            });
        }

        self.extract_play_info(&result, episode).ok_or_else(|| {
            PlayError::Other("player did not return any play information".to_string())
        })
    }

    // searches for shows with the given name and returns a list of ShowEdge
    fn get_shows(&self, show: String) -> Result<Vec<ShowEdge>, PlayError> {
        let gql = r#"
      query( $search: SearchInput, $limit: Int, $page: Int,
             $translationType: VaildTranslationTypeEnumType,
             $countryOrigin: VaildCountryOriginEnumType ) {
        shows(
          search: $search, limit: $limit, page: $page,
          translationType: $translationType, countryOrigin: $countryOrigin
        ) {
          edges { _id name availableEpisodes }
        }
      }"#;

        let variables = json!({
            "search": {"allowAdult": false, "allowUnknown": false, "query": show},
            "limit": 40,
            "page": 1,
            "translationType": "sub",
            "countryOrigin": "ALL"
        })
        .to_string();

        let headers = params![
            "User-Agent" => UA,
            "Referer" => REF,
        ];

        let params = params![
            "query" => gql,
            "variables" => variables,
        ];

        let result = send_request::<ShowSearch>("GET", API.to_string(), params, headers, None);
        match result {
            Ok(response) => {
                if response.data.shows.edges.is_empty() {
                    return Err(PlayError::NoResults("No shows found".to_string()));
                }
                Ok(response.data.shows.edges)
            }
            Err(e) => {
                Err(PlayError::Other(format!("Error fetching shows: {}", e)))
            }
        }
    }

    // finds the correct show id from the list of shows and returns its id
    fn extract_correct_id(&self, shows: &[ShowEdge], anime: &Anime) -> Result<String, PlayError> {
        // some functionality to get the correct show out of the list
        // TODO: actually add this functionality

        println!("Found {:?} shows matching \"{}\"", shows, anime.title);

        // try to match name exactly first:
        let show = shows.iter()
            .find(|s| s.name.eq_ignore_ascii_case(&anime.title))
            .or_else(|| shows.first())
            .ok_or(PlayError::NoResults(
                "No shows found".to_string(),
            ))?;

        println!("Playing \"{}\" ({}) episode: {}", show.name, show.id, anime.my_list_status.num_episodes_watched + 1);

        Ok(show.id.clone())
    }

    fn get_episode_providers(
        &self,
        show_id: &str,
        episode: u32,
    ) -> Result<Vec<SourceUrl>, PlayError> {
        let gql = r#"
        query($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) {
            episode(showId: $showId, translationType: $translationType, episodeString: $episodeString) {
                episodeString sourceUrls
            }
        }
      "#;

        let variables = json!({
            "showId": show_id,
            "translationType": "sub",
            "episodeString": episode.to_string(),
        })
        .to_string();

        let headers = params![
            "User-Agent" => UA,
            "Referer" => REF,
        ];

        let params = params![
            "query" => gql,
            "variables" => variables,
        ];

        let result = send_request::<EpisodeSearch>("GET", API.to_string(), params, headers, None);

        match result {
            Ok(mut response) => {
                if response.data.episode.source_urls.is_empty() {
                    return Err(PlayError::NoResults("No episodes found".to_string()));
                }

                for source in response.data.episode.source_urls.iter_mut() {
                    if source.source_url.starts_with("http") {
                        continue;
                    }

                    let (source_url, http_appended) = AnimePlayer::decode_clock(&source.source_url)
                        .unwrap_or_else(|_| (source.source_url.clone(), false));

                    source.source_url = source_url;

                    if !http_appended {
                        //this means the url already had https (its
                        //already its full path), nothing more to do (i think)
                        continue;
                    }

                    let headers = params![
                        "User-Agent" => UA,
                        "Referer" => REF,
                    ];

                    let url = source.source_url.clone();

                    if let Ok(link_details) =
                        send_request::<LinksSearch>("GET", url, params![], headers, None)
                    {
                        source.extra_values = link_details.links.into_iter().next();
                    }
                }

                Ok(response.data.episode.source_urls)
            }
            Err(e) => {
                Err(PlayError::Other(format!("Error fetching episodes: {}", e)))
            }
        }
    }

    fn decode_clock(enc: &str) -> Result<(String, bool), String> {
        let bytes = enc.trim_start_matches("--");
        if bytes.len() % 2 != 0 {
            return Err("odd-length clock encoding".into());
        }

        let mut out = String::with_capacity(bytes.len() / 2);
        for i in (0..bytes.len()).step_by(2) {
            let key = &bytes[i..i + 2].to_ascii_lowercase();
            let ch = match key.as_str() {
                "79" => "A",
                "7a" => "B",
                "7b" => "C",
                "7c" => "D",
                "7d" => "E",
                "7e" => "F",
                "7f" => "G",
                "70" => "H",
                "71" => "I",
                "72" => "J",
                "73" => "K",
                "74" => "L",
                "75" => "M",
                "76" => "N",
                "77" => "O",
                "68" => "P",
                "69" => "Q",
                "6a" => "R",
                "6b" => "S",
                "6c" => "T",
                "6d" => "U",
                "6e" => "V",
                "6f" => "W",
                "60" => "X",
                "61" => "Y",
                "62" => "Z",
                "59" => "a",
                "5a" => "b",
                "5b" => "c",
                "5c" => "d",
                "5d" => "e",
                "5e" => "f",
                "5f" => "g",
                "50" => "h",
                "51" => "i",
                "52" => "j",
                "53" => "k",
                "54" => "l",
                "55" => "m",
                "56" => "n",
                "57" => "o",
                "48" => "p",
                "49" => "q",
                "4a" => "r",
                "4b" => "s",
                "4c" => "t",
                "4d" => "u",
                "4e" => "v",
                "4f" => "w",
                "40" => "x",
                "41" => "y",
                "42" => "z",
                "08" => "0",
                "09" => "1",
                "0a" => "2",
                "0b" => "3",
                "0c" => "4",
                "0d" => "5",
                "0e" => "6",
                "0f" => "7",
                "00" => "8",
                "01" => "9",
                "15" => "-",
                "16" => ".",
                "67" => "_",
                "46" => "~",
                "02" => ":",
                "17" => "/",
                "07" => "?",
                "1b" => "#",
                "63" => "[",
                "65" => "]",
                "78" => "@",
                "19" => "!",
                "1c" => "$",
                "1e" => "&",
                "10" => "(",
                "11" => ")",
                "12" => "*",
                "13" => "+",
                "14" => ",",
                "03" => ";",
                "05" => "=",
                "1d" => "%",
                _ => return Err(format!("unknown code {key}")),
            };
            out.push_str(ch);
        }

        if out.ends_with("/clock") {
            out.push_str(".json");
        }
        // replace all occurrences
        else if !out.ends_with("/clock.json") {
            out = out.replace("/clock", "/clock.json");
        }

        // return the url if it already includes the host
        if out.starts_with("https://") || out.starts_with("http://") {
            return Ok((out, false));
        }

        Ok((BASE.to_string() + &out, true))
    }

    fn extract_best_candidate(
        &self,
        sources: &[SourceUrl],
    ) -> Result<(String, Option<String>), PlayError> {
        let mut variants: Vec<(i32, String, Option<String>, i32)> = Vec::new();

        for source in sources {
            let Some(link) = source
                .extra_values
                .as_ref()
                .map(|l| l.link.as_str())
                .filter(|s| !s.is_empty())
            else {
                variants.push((0, source.source_url.clone(), None, 0));
                continue;
            };

            if let Some(values) = self.convert_wixmp(link) {
                for (qlt, url) in values {
                    variants.push((qlt, url, None, 2))
                }
                continue;
            }

            if let Some(values) = self.parse_master_m3u8(link) {
                for (qlt, url) in values {
                    variants.push((qlt, url, Some(REF.to_string()), 3))
                }
                continue;
            }

            //anything else (like sharepoint?)
            variants.push((1, link.to_string(), None, 0));
        }

        if variants.is_empty() {
            return Err(PlayError::NoResults("No playable sources".to_string()));
        }

        // sort: by height desc then by kind weight (HLS > MP4 > Other)
        variants.sort_by(|a, b| (b.0, b.3).cmp(&(a.0, a.3)));

        let (_, url, referer, _k) = variants.remove(0);
        Ok((url, referer))
    }

    /// https://repackager.wixmp.com/video.wixstatic.com/video/<id>/,1080p,720p,480p,/mp4/file.mp4.urlset/master.m3u8
    fn convert_wixmp(&self, url: &str) -> Option<Vec<(i32, String)>> {
        if !url.contains("repackager.wixmp.com") {
            return None;
        }

        let base = url
            .trim_start_matches("https://repackager.wixmp.com/")
            .trim_end_matches(".urlset/master.m3u8");

        // capture the comma quality list between ".../<id>/" and "/mp4/"
        let caps = self.wixmp_regex.captures(base)?;
        let id = caps.get(1)?.as_str();
        let quality_list = caps.get(2)?.as_str();

        let qualities: Vec<&str> = quality_list.split(',').collect();
        if qualities.is_empty() {
            return None;
        }

        // replace each ",<something>" segment with the chosen quality
        let mut out = Vec::new();
        for q in qualities {
            let h = q.trim_end_matches('p').parse::<i32>().unwrap_or(0);
            let u = format!(
                "https://video.wixstatic.com/video/{}/{}/mp4/file.mp4",
                id, q
            );
            out.push((h, u));
        }

        out.sort_by(|a, b| b.0.cmp(&a.0));
        Some(out)
    }

    fn parse_master_m3u8(&self, url: &str) -> Option<Vec<(i32, String)>> {
        if !url.ends_with("master.m3u8") {
            return None;
        }

        let parameters = params![];
        let headers = params![
            "User-Agent" => UA,
            "Referer" => REF,
        ];
        let body: Option<&str> = None;
        let text =
            send_request_expect_text("GET", url.to_string(), parameters, headers, body).ok()?;

        if !text.contains("#EXTM3U") {
            return None;
        }

        let base = Url::parse(url).ok()?;
        let mut out: Vec<(i32, String)> = Vec::new();
        let mut pending_height = 0i32;
        let mut want_url_next = false;

        for raw in text.lines() {
            let line = raw.trim();

            if line.starts_with("#EXT-X-I-FRAME-STREAM-INF") {
                want_url_next = false;
                pending_height = 0;
                continue;
            }

            if line.starts_with("#EXT-X-STREAM-INF") {
                pending_height = self.parse_height_from_inf(line);
                want_url_next = true;
                continue;
            }

            if want_url_next && !line.is_empty() && !line.starts_with('#') {
                // resolve relative → absolute
                let abs = if let Ok(u) = Url::parse(line) {
                    u
                } else if let Ok(u) = base.join(line) {
                    u
                } else {
                    want_url_next = false;
                    pending_height = 0;
                    continue;
                };
                out.push((pending_height, abs.to_string()));
                want_url_next = false;
                pending_height = 0;
            }
        }

        if out.is_empty() {
            return None;
        }

        // sort highest first
        out.sort_by(|a, b| b.0.cmp(&a.0));
        Some(out)
    }

    fn parse_height_from_inf(&self, inf_line: &str) -> i32 {
        // parse RESOLUTION=WxH (case-insensitive) and return H
        let lower = inf_line.to_ascii_lowercase();
        if let Some(pos) = lower.find("resolution=") {
            let after = &lower[pos + "resolution=".len()..];
            let mut w = String::new();
            let mut h = String::new();
            let mut seen_x = false;
            for ch in after.chars() {
                if ch == ',' || ch == ' ' {
                    break;
                }
                if ch == 'x' {
                    seen_x = true;
                    continue;
                }
                if ch.is_ascii_digit() {
                    if !seen_x {
                        w.push(ch);
                    } else {
                        h.push(ch);
                    }
                } else {
                    break;
                }
            }
            if let Ok(n) = h.parse::<i32>() {
                return n;
            }
        }
        0
    }

    fn play_video_in_mpv(&self, info: &(String, Option<String>)) -> Result<String, PlayError> {
        let mut cmd = Command::new("mpv");

        if let Some(referer) = &info.1 {
            cmd.arg(format!("--referrer={}", referer));
        }

        let output = cmd
            .arg(&info.0)
            .output()
            .map_err(|e| {
                if e.kind() == ErrorKind::NotFound {
                    PlayError::NotFound("mpv is not installed or not found in PATH".to_string())
                } else {
                    PlayError::Other(format!("Error running mpv: \n{}", e))
                }
            })?;

        let messy_stdout = String::from_utf8_lossy(&output.stdout);
        let messy_stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = self.ansi_regex.replace_all(&messy_stdout, "").to_string();
        let stderr = self.ansi_regex.replace_all(&messy_stderr, "").to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        if !stderr.is_empty() && exit_code != 0 {
            if stderr.contains("No results found!") {
                return Err(PlayError::NoResults(stderr));
            } else {
                return Err(PlayError::CommandFailed {
                    stderr,
                    exit_code: output.status.code().unwrap_or(-1),
                    stdout,
                });
            }
        }

        Ok(stdout)
    }

    fn run_command(
        &self,
        command: &str,
        anime: &Anime,
        episode: u32,
        url: Option<&(String, Option<String>)>,
    ) -> Result<(), String> {
        let cmd = command 
            .replace("{title}", &anime.title)
            .replace("{episode}", &episode.to_string())
            .replace( "{url}", url.map(|u| u.0.as_str()).unwrap_or_default(),)
            .replace( "{referer}", url.and_then(|u| u.1.as_deref()).unwrap_or(""),)
            .replace( "{referrer}", url.and_then(|u| u.1.as_deref()).unwrap_or(""),);

        let status = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .status()
            .map_err(|e| format!("Failed to run hook: {}", e))?;

        #[cfg(windows)]
        let status = Command::new("cmd")
            .arg("/C")
            .arg(&cmd)
            .status()
            .map_err(|e| format!("Failed to run hook: {}", e))?;

        if !status.success() {
            return Err(format!("Hook exited with status: {:?}", status.code()));
        }

        Ok(())
    }
}
