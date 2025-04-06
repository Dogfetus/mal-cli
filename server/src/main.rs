#![allow(unreachable_code)]
#[macro_use]
extern crate rouille;
extern crate pkce;

use ureq::Agent;
use std::{env, time::Duration};
use anyhow::Result;
use oauth2::CsrfToken;
use rand::Rng;

struct ExpectedBody {
    code: String,
    code_verifier: String,
}

struct MalAgent {
    url: String,
    agent: Agent,
    client_id: String,
    client_secret: String,
    redirect_url: String,
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

        MalAgent { url, agent, client_id, client_secret, redirect_url }
    }

    fn get_user_tokens(&self, data: ExpectedBody) -> Result<String> {
        let body = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", data.code.as_str()),
            ("redirect_uri", self.redirect_url.as_str()),
            ("code_verifier", data.code_verifier.as_str()),
        ];


        let response: String = self.agent.post(&self.url)
            .send_form(body)?
            .body_mut()
            .read_to_string()?;


        Ok(response)
    }

    fn get_oauth_url(&self) -> Result<String> {
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

        Ok(url)
    }
}


fn main() {
    dotenvy::dotenv().ok();

    let mal_url = "https://myanimelist.net/v1/oauth2/token".to_string();
    let mal_agent = MalAgent::new(mal_url);

    println!("Now listening on localhost:8000");

    rouille::start_server("0.0.0.0:8000", move |request| {
        router!(request,
            (GET) (/) => {
                rouille::Response::redirect_302("/hello")
            },

            (POST) (/auth/token) => {
                let data = try_or_400!(post_input!(request, {
                    code: String,
                    code_verifier: String,
                }));

                // idk if rouille can do this automatically
                // TODO: check this later
                let info = ExpectedBody {
                    code: data.code,
                    code_verifier: data.code_verifier,
                };

                mal_agent.get_user_tokens(info).unwrap();
                rouille::Response::text("hello")
            },

            (GET) (/oauth_url) => {
                let url = mal_agent.get_oauth_url().unwrap();
                rouille::Response::text(url)
            },

            _ => rouille::Response::empty_404()
        )
    });
}


