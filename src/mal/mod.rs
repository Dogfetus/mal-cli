mod oauth;
pub mod models;

use std::{fs, thread::JoinHandle};
use ureq;
use serde_json::{Value, json};
use models::anime::{self, Anime, fields::*};
use std::sync::{Arc, RwLock};



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

    pub fn log_out() {
        fs::remove_file(".mal/client").expect("Failed to remove client file");
    }

    pub fn test(&self) -> Result<Vec<Anime>, Box<dyn std::error::Error>> {
        let token = self.identity.read().unwrap().access_token.clone();
        if token.is_none() {
            return Err("Access token is not set".into());
        }
        let body = ureq::get("https://api.myanimelist.net/v2/anime")
            .header("Authorization", format!("Bearer {}", token.as_ref().unwrap()))
            .query("fields", &ALLFIELDS.join(","))
            .query("q", "one")
            .query("limit", "2")
            .call()?
            .body_mut()
            .read_to_string()?;

        let parsed: serde_json::Value = serde_json::from_str(&body)?;
        let animes = Anime::from_body(&parsed);

        Ok(animes)
    }
}

