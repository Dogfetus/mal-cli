mod oauth;
use std::{fs, thread::JoinHandle};
use ureq;
use serde_json::{Value, json};
use crate::models::anime::{self, Anime};


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



    pub fn test(&self) -> Result<(), Box<dyn std::error::Error>> {
        let fields = anime::fields::ALL;
        let body = ureq::get("https://api.myanimelist.net/v2/anime/season/2025/fall")
            .header("Authorization", format!("Bearer {}", self.access_token.as_ref().unwrap()))
            .query("fields", &fields.join(","))
            .query("limit", "1")
            .call()?
            .body_mut()
            .read_to_string()?;

        let parsed: serde_json::Value = serde_json::from_str(&body)?;


        let list = parsed["data"].as_array().expect("Failed to get data");
        // println!("List: {:?}", list);
        for item in list {
            // Get the "node" object which contains the actual anime data
            if let Some(node) = item.get("node") {
                // println!("Node: {:?}", node);
                for field in fields {
                    if let Some(value) = node.get(field) {
                        // println!("{}: {:?}", field, value);
                    } else {
                        println!("{}: None", field);
                    }
                }
            } else {
                println!("No 'node' field found in item");
            }
        }

        // just print whatever we got
        Ok(())
    }
}

