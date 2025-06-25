use cached::proc_macro::cached;
use std::io::Read;
use ureq::Error;
use ureq;

#[cached(size = 2000, result = true)]
pub fn fetch_image(url: String) -> Result<image::DynamicImage, String> {
    match ureq::get(&url).call() {
        Ok(mut response) => {
            let mut reader = response.body_mut().as_reader();
            let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer)
                .map_err(|e| e.to_string())?;

            image::load_from_memory(&buffer)
                .map_err(|e| e.to_string())
        },
        Err(Error::StatusCode(code)) => {
            Err(format!("HTTP error: {}", code))
        }
        Err(e) => {
            Err(format!("Request failed: {}", e))
        }
    }
}
