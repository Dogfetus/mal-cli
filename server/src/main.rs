#![allow(unreachable_code)]
#[macro_use]
extern crate rouille;
extern crate pkce;

use ureq::Agent;
use chrono::Local;
use std::{collections::HashMap, env};
use anyhow::Result;
use oauth2::CsrfToken;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use rand::Rng;
use std::thread;
use std::io::Write;

const STATE_LIFETIME: u64 = 300; // 5 minutes
const CLEANUP_INTERVAL: u64 = 30; // 30 seconds


struct ExpectedBody {
    code: String,
    state: String,
}

struct Data {
    code_challenge: String,
    port: u16,
    timestamp: Instant,
}

struct MalAgent {
    url: String,
    agent: Agent,
    client_id: String,
    client_secret: String,
    redirect_url: String,
    temp_storage: HashMap<String, Data>,
}

impl MalAgent {
    fn new(url: String) -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .build();

        let agent: Agent = config.into();

        let client_id = env::var("MAL_CLIENT_ID").unwrap();
        let client_secret = env::var("MAL_CLIENT_SECRET").unwrap();
        let redirect_url = env::var("MAL_REDIRECT_URL").unwrap();
        let temp_storage = HashMap::new();

        MalAgent { url, agent, client_id, client_secret, redirect_url, temp_storage }
    }

    fn get_user_tokens(&self, data: &ExpectedBody) -> Result<String> {
        let storage = match self.temp_storage.get(&data.state){
            Some(storage) => storage,
            None => {
                println!("No data found for state: {}", data.state);
                return Err(anyhow::anyhow!("No data found for state"));
            }
        };


        let body = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", data.code.as_str()),
            ("redirect_uri", self.redirect_url.as_str()),
            ("code_verifier", storage.code_challenge.as_str()),
        ];


        let response: String = self.agent.post(&self.url)
            .send_form(body)?
            .body_mut()
            .read_to_string()?;

        Ok(response)
    }

    fn refresh_user_tokens(&self, refresh_token: String) -> Result<String>{
        const URL: &str = "https://myanimelist.net/v1/oauth2/token";

        body = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token)
        ];

        let response: String = self.agent
        .post(URL)
        .send_form(&body)?
        .body_mut()
        .read_to_string()?;

        Ok(response)
    }

    fn get_oauth_url(&self) -> Result<(String, String, String)> {
        const URL: &str = "https://myanimelist.net/v1/oauth2/authorize";

        let mut rng = rand::rng();
        let n: usize = rng.random_range(48..=128);
        let csr = CsrfToken::new_random();
        let state = csr.secret().to_string();
        let code_verify = pkce::code_verifier(n);
        let code_challenge = pkce::code_challenge(&code_verify);

        let url = format!("{}?\
                response_type=code\
                &client_id={}\
                &state={}\
                &code_challenge={}\
                &code_challenge_method=plain\
                &redirect_uri={}
            ",
            URL,
            self.client_id,
            state,
            code_challenge,
            self.redirect_url
        );

        Ok((url, state, code_challenge))
    }

    fn save_data(&mut self, state: String, code_challenge: String, port: u16) {
        let data = Data {
            code_challenge, 
            port,
            timestamp: Instant::now(),
        };

        self.temp_storage.insert(state, data);
    }

    fn remove_data(&mut self, state: &String) {
        self.temp_storage.remove(state);
    }

    fn cleanup_expired_data(&mut self, max_age: Duration) -> usize {
        let now = Instant::now();
        let expired_keys: Vec<String> = self.temp_storage
            .iter()
            .filter(|(_, data)| now.duration_since(data.timestamp) > max_age)
            .map(|(key, _)| key.clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            self.temp_storage.remove(&key);
        }
        count
    }

    fn handle_token_response(&mut self, result: Result<String>, data: &ExpectedBody) -> (String, u16) {
        match result {
            Ok(response) => {
                let port = self.temp_storage[&data.state].port;
                let local_url = format!("http://localhost:{}/callback", port);
                let json: serde_json::Value = match serde_json::from_str(&response) {
                    Ok(json) => json,
                    Err(_) => {
                        return ("Invalid JSON response".to_string(), 500) 
                    }
                };
                let token = match json["access_token"].as_str() {
                    Some(token) => token,
                    None => {
                        return ("Missing access_token".to_string(), 500) 
                    }
                };
                let refresh_token = match json["refresh_token"].as_str() {
                    Some(token) => token,
                    None => {
                        return ("Missing refresh_token".to_string(), 500) 
                    }
                };
                let expires_in = match json["expires_in"].as_u64() {
                    Some(token) => token,
                    None => {
                        return ("Missing expires_in".to_string(), 500) 
                    }
                };


                self.remove_data(&data.state);

                // hmmmm>
                let mut html_content = match std::fs::read_to_string("templates/success.html") {
                    Ok(content) => content,
                    Err(_) => return ("Failed to read template".to_string(), 400) 
                };
                html_content = html_content.replace("{{redirect_url}}", &local_url)
                                        .replace("{{access_token}}", &token)
                                        .replace("{{refresh_token}}", &refresh_token)
                                        .replace("{{expires_in}}", &expires_in.to_string());

                return (html_content, 200)
            }


            //ERROR: s  
            Err(e) => {
                let mut html_content = match std::fs::read_to_string("templates/error.html") {
                    Ok(content) => content,
                    Err(_) => return ("Failed to read template".to_string(), 400)
                };

                html_content = html_content.replace("{{error}}", &e.to_string());
                return (html_content, 500)
            }
        }
    }
}



// TODO: check for different errors (unexpected input)
fn main() {
    dotenvy::dotenv().ok();

    let mal_url = "https://myanimelist.net/v1/oauth2/token".to_string();
    let mal_agent = Arc::new(Mutex::new(MalAgent::new(mal_url)));
    let cleanup_agent = Arc::clone(&mal_agent);


    // cleanup thread
    thread::spawn(move || {
        let max_age = Duration::from_secs(STATE_LIFETIME);
        loop {
            thread::sleep(Duration::from_secs(CLEANUP_INTERVAL));
            if let Ok(mut guard) = cleanup_agent.lock() {
                let removed = guard.cleanup_expired_data(max_age);
                let now = Local::now().format("%Y-%m-%d %H:%M:%S");
                println!("[{}] Cleaned up {} expired states, {} states remaining",
                    now,
                    removed, 
                    guard.temp_storage.len()
                );
                std::io::stdout().flush().unwrap();
            }
        }
    });


    // server
    println!("Now listening on localhost:8000");
    rouille::start_server("0.0.0.0:8000", move |request| {
        router!(request,
            (GET) (/) => {
                rouille::Response::text("hello")
            },



            (POST) (/oauth_url) => {
                let data = try_or_400!(post_input!(request, {
                    port: u16,
                }));
                let mut guard = mal_agent.lock().unwrap();
                let (url, state, code_challenge) = guard.get_oauth_url().unwrap();

                guard.save_data(state, code_challenge, data.port);
                rouille::Response::text(url)
            },



            (POST) (/refresh_token) => {
                let data = try_or_400!(post_input!(request, {
                    refresh_token: u16,
                }));

                let mut guard = mal_agent.lock().unwrap();
                let result = guard.refresh_user_tokens(refresh_token);

                rouille::Response::text(result)
            }



            (GET) (/callback) => {
                let mut guard = mal_agent.lock().unwrap();
                let code = match request.get_param("code") {
                    Some(code) => code,
                    None => return rouille::Response::text("Missing code parameter").with_status_code(400)
                };
                let state = match request.get_param("state") {
                    Some(state) => state,
                    None => return rouille::Response::text("Missing state parameter").with_status_code(400)
                };
                let info = ExpectedBody { code, state };
                let result = guard.get_user_tokens(&info);
                let (html, status_code) = guard.handle_token_response(result, &info);

                if status_code == 200 {
                    println!("Successfull login");
                }

                rouille::Response::html(html).with_status_code(status_code)
            },

            _ => rouille::Response::text("Nothing here...").with_status_code(404)
        // 
        )
    });
}
