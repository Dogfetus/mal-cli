mod oauth;
use std::{fs, thread::JoinHandle};
use ureq;
use serde_json::{Value, json};


//TODO: idk where to place this callback function 
//TODO: should this be oauth login or just login with options?
//TODO: startup screen should include an option to signin or not (or profile side)
//TODO: encrypt the tokens somehow
//TODO: check if the tokens exists before trying to login 
//TODO: read the tokens to memory, and start using them to request data (using some mal api wrapper)


pub struct MalClient {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<String>,
}

impl MalClient {
    pub fn new() -> Self {
        let mut client = Self {
            access_token: None,
            refresh_token: None,
            expires_in: None,
        };

        client.load_from_file();
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

    pub fn load_from_file(&mut self) -> bool {
        if !fs::metadata(".mal/client").is_ok() {
            return false;
        }

        if let Ok(client_file) = fs::read_to_string(".mal/client") {
            for line in client_file.lines() {
                if line.starts_with("mal_access_token") {
                    self.access_token = line.split("\"").nth(1).map(String::from);
                } else if line.starts_with("mal_refresh_token") {
                    self.refresh_token = line.split("\"").nth(1).map(String::from);
                } else if line.starts_with("mal_token_expires_at") {
                    self.expires_in = line.split("\"").nth(1).map(String::from);
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

    // fields:
    // Basic: id, title, main_picture, alternative_titles, start_date, end_date, synopsis
    // Details: mean (score), rank, popularity, num_list_users, num_episodes, status
    // Media: pictures, background, related_anime, related_manga
    // Content: genres, studios, recommendations
    // Community: statistics, my_list_status, broadcast, opening_themes, ending_themes


    pub fn test(&self) -> Result<(), Box<dyn std::error::Error>>{
        let body: String = ureq::get("https://api.myanimelist.net/v2/anime/season/2025/fall")
        .header("Authorization", format!("Bearer {}", self.access_token.as_ref().unwrap()))
        .query("limit", "10")
        .query("offset", "0")
        .query("fields", "id,title,main_picture,genres,start_date,end_date,synopsis")
        .query("sort", "start_date")
        .call()?
        .body_mut()
        .read_to_string()?;

        let parsed: Value = serde_json::from_str(&body)?;
        println!("here:\n{}", serde_json::to_string_pretty(&parsed)?);
        Ok(())
    }
}

