use super::models::anime::{AnimeResponse, FavoriteResponse};
use super::models::user::User;
use cached::proc_macro::cached;
use std::fmt::Debug;
use std::io::Read;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use ureq::{Agent, Error};
use url::Url;

#[macro_export]
macro_rules! params {
    ($($key:expr => $value:expr),* $(,)?) => {
        vec![$(($key.to_string(), $value.to_string())),*]
    };
}

const PROXY: &str = "http://localhost:1111/proxy?url=";
const MAX_RETRIES: u32 = 5;
static AGENT: OnceLock<Agent> = OnceLock::new();
fn get_agent() -> &'static Agent {
    AGENT.get_or_init(|| {
        Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .build()
            .into()
    })
}

#[cached(size = 2000, result = true)]
pub fn fetch_image(uri: String) -> Result<image::DynamicImage, String> {
    let url = Url::parse(&uri).map_err(|e| format!("Invalid URL: {}", e))?;

    let agent = get_agent();

    match url.scheme() {
        "http" | "https" => loop {
            match agent.get(&format!("{}{}", PROXY, uri)).call() {
                Ok(mut response) => {
                    let mut reader = response.body_mut().as_reader();
                    let mut buffer = Vec::new();
                    reader.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

                    return image::load_from_memory(&buffer).map_err(|e| e.to_string());
                }
                Err(Error::StatusCode(code)) => return Err(format!("HTTP error: {}", code)),
                Err(e) => {
                    let error_message = e.to_string().to_lowercase();
                    let error_is_timeout =
                        error_message.contains("timeout") || error_message.contains("timed out");

                    if !error_is_timeout {
                        return Err(format!("Request failed: {}", e));
                    }
                }
            }
        },
        "file" => {
            let path = url
                .to_file_path()
                .map_err(|_| "Invalid file URL".to_string())?;
            image::open(path).map_err(|e| e.to_string())
        }
        _ => return Err("Unsupported URL scheme".to_string()),
    }
}

#[cached(size = 2000, result = true)]
pub fn fetch_anime(
    token: String,
    url: String,
    parameters: Vec<(String, String)>,
) -> Result<AnimeResponse, Box<dyn std::error::Error>> {
    send_request::<AnimeResponse>(token, url, parameters)
}

#[cached(result = true)]
pub fn fetch_user(
    token: String,
    url: String,
    parameters: Vec<(String, String)>,
) -> Result<User, Box<dyn std::error::Error>> {
    send_request::<User>(token, url, parameters)
}

#[cached(result = true)]
pub fn fetch_favorited_anime(
    token: String,
    url: String,
    parameters: Vec<(String, String)>,
) -> Result<FavoriteResponse, Box<dyn std::error::Error>> {
    send_request::<FavoriteResponse>(token, url, parameters)
}


fn build_url(
    base_url: &str,
    parameters: &[(String, String)],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut url = Url::parse(base_url)?;

    for (key, value) in parameters {
        url.query_pairs_mut().append_pair(key, value);
    }

    let target_url = url.to_string();
    Ok(format!("{}{}", PROXY, target_url))
}

// not cacheable since T
pub fn send_request<T>(
    token: String,
    url: String,
    parameters: Vec<(String, String)>,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned + Debug,
{
    if token.is_empty() {
        return Err("Access token is not set".into());
    }

    let final_url = build_url(&url, &parameters)
        .map_err(|e| format!("Failed to build proxied URL: {}", e))?;

    let agent = get_agent();
    for attempt in 0..MAX_RETRIES {
        // create request
        let request = agent
            .get(&final_url)
            .header("Authorization", format!("Bearer {}", token));

        // check for errors
        match request.call() {
            // all good
            Ok(mut response) => return Ok(response.body_mut().read_json::<T>()?),

            // request successful but with an error status code
            Err(ureq::Error::StatusCode(status)) => {
                return Err(format!("HTTP error: {}", status).into());
            }

            // request failed due to network error or timeout etc
            Err(e) => {
                let error_message = e.to_string().to_lowercase();
                let error_is_timeout =
                    error_message.contains("timeout") || error_message.contains("timed out");

                if !error_is_timeout {
                    return Err(format!("Request failed: {}", e).into());
                }

                if attempt >= MAX_RETRIES - 1 {
                    return Err(format!("max retries exceeded: {}, {}", MAX_RETRIES, e).into());
                }

                println!("Attempt {}: Request failed with error: {}", attempt + 1, e);

                let delay = Duration::from_millis(2000 * (attempt + 1) as u64);
                thread::sleep(delay);
            }
        }
    }

    Err("All retry attempts failed".into())
}

pub trait Fetchable: Sized {
    type Response;
    type Output;

    fn fetch(
        token: String,
        url: String,
        parameters: Vec<(String, String)>,
    ) -> Result<Self::Response, Box<dyn std::error::Error>>;

    fn from_response(response: Self::Response) -> Self::Output;
}
