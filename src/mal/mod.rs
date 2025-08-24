pub mod models;
pub mod network;
mod oauth;

use crate::mal::network::Fetchable;
use crate::params;
use crate::utils::get_app_dir;
use chrono::{Datelike, Local};
use models::anime::{Anime, AnimeId, FavoriteAnime, fields};
use models::user::User;
use network::Update;
use regex::Regex;
use std::any::type_name;
use std::sync::{Arc, RwLock};
use std::{fs, thread::JoinHandle};

const BASE_URL: &str = "https://api.myanimelist.net/v2";
const EXTRA_URL: &str = "https://api.jikan.moe/v4";
const CLIENT_FOLDER: &str = ".mal";
const CLIENT_FILE: &str = "client";

//TODO: encrypt the tokens
#[derive(Debug, Clone)]
pub struct Identity {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MalClient {
    identity: Arc<RwLock<Identity>>,
    re: Regex,
}

impl MalClient {
    pub fn new() -> Self {
        let client = Self {
            identity: Arc::new(RwLock::new(Identity {
                access_token: None,
                refresh_token: None,
                expires_in: None,
            })),
            re: Regex::new(r"\(([0-9,]+)/([0-9,]+|Unknown)\)").unwrap(),
        };

        client.login_from_file();
        client
    }

    pub fn init_oauth() -> (String, JoinHandle<()>) {
        oauth::oauth_login(|at, rt, ei| {
            // format the tokens and expiration time
            let data = format!(
                "mal_access_token = \"{}\"\nmal_refresh_token = \"{}\"\nmal_token_expires_at = \"{}\"",
                at, rt, ei
            );

            // get the file path and folder
            let app_dir = get_app_dir();
            let mal_dir = app_dir.join(CLIENT_FOLDER);
            if !mal_dir.exists() {
                fs::create_dir_all(&mal_dir).expect("Failed to create app directory");
            }

            // write the data to the client file
            let client_file = mal_dir.join("client");
            fs::write(client_file, data)?;
            Ok(())
        })
    }

    //TODO: add a check for token validity
    pub fn login_from_file(&self) -> bool {
        let app_dir = get_app_dir();
        if !app_dir.exists() || !app_dir.join(format!("{}/{}", CLIENT_FOLDER, CLIENT_FILE)).exists() {
            return false;
        }

        if let Ok(client_file) = fs::read_to_string(app_dir.join(format!("{}/{}", CLIENT_FOLDER, CLIENT_FILE))) {
            let mut identity = self.identity.write().unwrap();
            for line in client_file.lines() {
                if line.starts_with("mal_access_token") {
                    identity.access_token = line.split("\"").nth(1).map(String::from);
                } else if line.starts_with("mal_refresh_token") {
                    identity.refresh_token = line.split("\"").nth(1).map(String::from);
                } else if line.starts_with("mal_token_expires_at") {
                    identity.expires_in = line.split("\"").nth(1).map(String::from);
                }
            }
            return true;
        }
        false
    }

    pub fn update_user_login(&self) {
        self.login_from_file();
    }

    pub fn log_out() {
        let app_dir = get_app_dir();
        if !app_dir.exists() || !app_dir.join(format!("{}/{}", CLIENT_FOLDER, CLIENT_FILE)).exists() {
            return;
        }
        fs::remove_file(app_dir.join(format!("{}/{}", CLIENT_FOLDER, CLIENT_FILE))).expect("Failed to remove client file");
    }

    pub fn user_is_logged_in() -> bool {
        let app_dir = get_app_dir();
        let client_file = app_dir.join(format!("{}/{}", CLIENT_FOLDER, CLIENT_FILE));

        if !client_file.exists() {
            return false;
        }

        if let Ok(content) = fs::read_to_string(&client_file) {
            return content.contains("mal_access_token");
        }

        false
    }

    pub fn current_season() -> (u16, String) {
        let now = Local::now();
        let year = now.year() as u16;
        let month = now.month();

        let season = match month {
            1 | 2 | 3 => "winter",
            4 | 5 | 6 => "spring",
            7 | 8 | 9 => "summer",
            _ => "fall",
        };

        (year, season.to_string())
    }

    pub fn get_seasonal_anime(
        &self,
        year: u16,
        season: String,
        offset: usize,
        limit: usize,
    ) -> Option<Vec<Anime>> {
        self.send_request::<Anime>(
            format!(
                "{}/anime/season/{}/{}",
                BASE_URL,
                year,
                season.to_lowercase()
            ),
            params![
               "fields" => fields::ALL.join(","),
                "limit" => limit,
                "offset" => offset,
                "sort" => "anime_num_list_users",
                "nsfw" => "true",
            ],
        )
    }

    pub fn get_suggested_anime(&self, offset: usize, limit: usize) -> Option<Vec<Anime>> {
        self.send_request::<Anime>(
            format!("{}/anime/suggestions", BASE_URL),
            params![
                "fields" => fields::ALL.join(","),
                "limit" => limit,
                "offset" => offset,
                "nsfw" => "true",
            ],
        )
    }

    pub fn get_top_anime(&self, filter: String, offset: usize, limit: usize) -> Option<Vec<Anime>> {
        self.send_request::<Anime>(
            format!("{}/anime/ranking", BASE_URL),
            params![
            "ranking_type" => filter,
            "fields" => fields::ALL.join(","),
            "limit" => limit,
            "offset" => offset,
            "nsfw" => "true",
            ],
        )
    }

    pub fn search_anime(&self, query: String, offset: usize, limit: usize) -> Option<Vec<Anime>> {
        self.send_request::<Anime>(
            format!("{}/anime", BASE_URL),
            params![
                "q" => query,
                "fields" => fields::ALL.join(","),
                "limit" => limit,
                "offset" => offset,
                "nsfw" => "true",
            ],
        )
    }

    pub fn get_user(&self) -> Option<User> {
        self.send_request::<User>(
            format!("{}/users/@me", BASE_URL),
            params![
                "fields" => "anime_statistics",
                "nsfw" => "true",
            ],
        )
    }

    pub fn get_anime_list(
        &self,
        status: Option<String>,
        offset: usize,
        limit: usize,
    ) -> Option<Vec<Anime>> {
        self.get_anime_list_by_user("@me".to_string(), status, offset, limit)
    }

    pub fn get_anime_list_by_user(
        &self,
        username: String,
        status: Option<String>,
        offset: usize,
        limit: usize,
    ) -> Option<Vec<Anime>> {
        let mut parameters = params![
            "fields" => fields::ALL.join(","),
            "limit" => limit,
            "offset" => offset,
            "sort" => "list_updated_at",
            "nsfw" => "true",
        ];

        if let Some(status) = status {
            parameters.push(("status".to_string(), status));
        }

        self.send_request::<Anime>(
            format!("{}/users/{}/animelist", BASE_URL, username),
            parameters,
        )
    }

    pub fn get_favorited_anime(&self, username: String) -> Option<Vec<FavoriteAnime>> {
        self.send_request::<FavoriteAnime>(
            format!("{}/users/{}/favorites", EXTRA_URL, username),
            params![],
        )
    }

    pub fn update_user_list<T: Update>(
        &self,
        element: T,
    ) -> Result<(usize, T::Response), Box<(dyn std::error::Error + 'static)>> {
        let token = self.identity.read().unwrap().access_token.clone();
        if token.is_none() {
            eprintln!("User is not logged in. Cannot send request.");
            return Err("not logged in".into());
        }

        element.update(
            token.unwrap(),
            format!(
                "{}/{}/{}/my_list_status",
                BASE_URL,
                element.get_belonging_list(),
                element.get_id()
            ),
        )
    }

    pub fn update_user_list_async<T: Update + Send + 'static>(
        &self,
        element: T,
    ) -> tokio::task::JoinHandle<
        Result<(usize, T::Response), Box<(dyn std::error::Error + Send + 'static)>>,
    >
    where
        T::Response: Send,
    {
        let client = self.clone();
        tokio::task::spawn_blocking(move || {
            client.update_user_list(element).map_err(
                |e| -> Box<dyn std::error::Error + Send + 'static> {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("{}", e),
                    ))
                },
            )
        })
    }

    // this a very specific request i must say (gets the number of available episodes for an anime)
    pub fn get_available_episodes(
        &self,
        anime_id: AnimeId,
    ) -> Result<Option<u32>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://myanimelist.net/anime/{}/thiscanbewhatever/episode",
            anime_id
        );
        let mut response = ureq::get(&url).call()?;
        let html = response.body_mut().read_to_string()?;
        if let Some(captures) = self.re.captures(&html) {
            if let Some(available_str) = captures.get(1) {
                let cleaned = available_str.as_str().replace(",", "");
                return Ok(Some(cleaned.parse::<u32>()?));
            }
        }
        Ok(None)
    }

    fn send_request<T>(&self, url: String, parameters: Vec<(String, String)>) -> Option<T::Output>
    where
        T: Fetchable,
    {
        let token = self.identity.read().unwrap().access_token.clone();
        if token.is_none() {
            eprintln!("User is not logged in. Cannot send request.");
            return None;
        }

        let response = T::fetch(token.unwrap(), url, parameters);
        let response = match response {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Error fetching {}: {:?}", type_name::<T>(), e);
                return None;
            }
        };
        Some(T::from_response(response))
    }
}
