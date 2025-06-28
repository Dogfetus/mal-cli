use super::models::anime::AnimeResponse;
use cached::proc_macro::cached;
use std::io::Read;
use ureq;
use ureq::Error;

#[macro_export]
macro_rules! params {
    ($($key:expr => $value:expr),* $(,)?) => {
        vec![$(($key.to_string(), $value.to_string())),*]
    };
}

#[cached(size = 2000, result = true)]
pub fn fetch_image(url: String) -> Result<image::DynamicImage, String> {
    match ureq::get(&url).call() {
        Ok(mut response) => {
            let mut reader = response.body_mut().as_reader();
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

            image::load_from_memory(&buffer).map_err(|e| e.to_string())
        }
        Err(Error::StatusCode(code)) => Err(format!("HTTP error: {}", code)),
        Err(e) => Err(format!("Request failed: {}", e)),
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

pub fn send_request<T>(
    token: String,
    url: String,
    parameters: Vec<(String, String)>,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    if token.is_empty() {
        return Err("Access token is not set".into());
    }

    let mut request = ureq::get(url).header("Authorization", format!("Bearer {}", token));

    for (key, value) in parameters {
        request = request.query(&key, &value);
    }

    let response = request.call()?.body_mut().read_json::<T>()?;

    Ok(response)
}
