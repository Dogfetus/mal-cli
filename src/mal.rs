use ureq::Agent;
use std::{env, time::Duration};
use anyhow::Result;


pub fn test() -> Result<String> {
    let url = env::var("URL")?;

    let config = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();

    let agent: Agent = config.into();


    let body: String = agent.get(url)
        .call()?
        .body_mut()
        .read_to_string()?;


    // Reuses the connection from previous request.
    // let response: String = agent.put("http://example.com/upload")
    //     .header("Authorization", "example-token")
    //     .send("some body data")?
    //     .body_mut()
    //     .read_to_string()?;

    Ok(body) 
}
