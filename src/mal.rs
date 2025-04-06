use ureq;
use anyhow::Result;

static BACKEND_URL: &str = "http://localhost:8000";


// TODO: add functionality to change port number for localhost
fn get_oauth_url() -> Result<String> {
    let full_url = format!("{}/oauth_url", BACKEND_URL); 
    let url = ureq::get(&full_url)
        .call()?
        .body_mut()
        .read_to_string()?;
    println!("get_oauth_url{}", url);
    Ok(url)
}

pub async fn oauth_login() {
    get_oauth_url().unwrap();
}
