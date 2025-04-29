mod oauth;
use std::{fs, thread::JoinHandle};

//TODO: idk where to place this callback function 
//TODO: should this be oauth login or just login with options?
//TODO: startup screen should include an option to signin or not (or profile side)
//TODO: encrypt the tokens somehow
//TODO: check if the tokens exists before trying to login 
//TODO: read the tokens to memory, and start using them to request data (using some mal api wrapper)
//p
pub fn init_oauth() -> (String, JoinHandle<()>) {
    if !fs::metadata(".mal").is_ok() {
        fs::create_dir(".mal").expect("Failed to create .mal directory");
    }

    oauth::oauth_login( |at, rt, ei| 
        {
            let data = format!("Access Token: \"{}\"\nRefresh Token: \"{}\"\nExpires In: \"{}\"", at, rt, ei);
            fs::write(".mal/client", data)?;
            Ok(())
        })
}


