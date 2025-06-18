mod oauth;
pub mod models;

use std::{fs, thread::JoinHandle};
use ureq;
use serde_json::{Value, json};
use models::anime::{self, fields, Anime, AnimeResponse};
use std::sync::{Arc, RwLock};
use chrono::{Datelike, Local};


const BASE_URL: &str = "https://api.myanimelist.net/v2";


//TODO: encrypt the tokens somehow???

#[derive(Debug, Clone)]
pub struct Identity {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<String>,
}


#[derive(Debug, Clone)]
pub struct MalClient {
    identity: Arc<RwLock<Identity>>,
}

impl MalClient {
    pub fn new() -> Self {
        let mut client = Self {
            identity: Arc::new(RwLock::new(Identity {
                access_token: None,
                refresh_token: None,
                expires_in: None,
            })),
        };

        client.login_from_file();
        client
    }

    pub fn init_oauth() -> (String, JoinHandle<()>) {
        if !fs::metadata(".mal").is_ok() {
            fs::create_dir(".mal").expect("Failed to create .mal directory");
        }

        oauth::oauth_login( |at, rt, ei| 
            {
                let data = format!("mal_access_token = \"{}\"\nmal_refresh_token = \"{}\"\nmal_token_expires_at = \"{}\"", at, rt, ei);
                fs::write(".mal/client", data)?;
                Ok(())
            }
        )
    }


    //TODO: add a check for token validity
    pub fn login_from_file(&mut self) -> bool {
        if !fs::metadata(".mal/client").is_ok() {
            return false;
        }


        if let Ok(client_file) = fs::read_to_string(".mal/client") {
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

    pub fn log_out() {
        fs::remove_file(".mal/client").expect("Failed to remove client file");
    }

    pub fn user_is_logged_in() -> bool {
        let client_file = fs::metadata(".mal/client");
        if client_file.is_ok() {
            let client_file = fs::read_to_string(".mal/client").expect("Failed to read client file");
            if client_file.contains("mal_access_token") {
                return true;
            }
        }
        false
    }

    pub fn get_current_season(&self, offset: u16, limit: u16) -> Option<Vec<Anime>> {
        let (year, season) = Self::current_season();
        self.get_seasonal_anime(year, season, offset, limit)
    }

    pub fn current_season() ->  (u16, String) {
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

    pub fn get_seasonal_anime(&self, year: u16, season: String, offset: u16, limit: u16) -> Option<Vec<Anime>> {
        let anime_response = self.send_request::<AnimeResponse>(&format!("{}/anime/season/{}/{}", BASE_URL, year, season), 
            &[
                ("fields", &fields::ALL.join(",")),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
                ("sort", "anime_num_list_users"),
            ],
        );
        let response = match anime_response {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Error fetching seasonal anime: {}", e);
                return None;
            }
        };
        let animes = Anime::from_response(response);
        Some(animes)
    }

    fn send_request<T>(&self, url: &str, parameters: &[(&str, &str)]) -> Result<T, Box<dyn std::error::Error>> 
    where
        T: serde::de::DeserializeOwned,
    {

        let token = self.identity.read().unwrap().access_token.clone();
        if token.is_none() {
            return Err("Access token is not set".into());
        }

        let mut request = ureq::get(url)
            .header("Authorization", format!("Bearer {}", token.as_ref().unwrap()));

        for (key, value) in parameters {
            request = request.query(&key, &value);
        }

        let response = request.call()?
            .body_mut()
            .read_json::<T>()?;

        Ok(response)
    }
}

