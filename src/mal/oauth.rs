#![allow(unreachable_code)]
use ureq;
use rouille::router;
use anyhow::Result;
use std::thread;
use std::time::Duration;
use rouille::try_or_400;
use rouille::post_input;



const MAX_RETRIES: u16 = 10;
const BACKEND_URL: &str = "https://mal-cli.dogfetus.no";



//TODO: this might be moved to mod.rs instead (this function)
//TODO: might change to sending a callback function to wait for redirect 
//TODO: add thread that stops the server when its gotten the data (callback is called) 
pub fn oauth_login() {
    let port = wait_for_redirect();
    let url = get_oauth_url(port).expect("Can't connect to backend");
    println!("url: {}", url);
}


fn get_oauth_url(port: u16) -> Result<String> {
    let full_url = format!("{}/oauth_url", BACKEND_URL); 
    let body = [ ("port", port.to_string()) ];
    let url: String = ureq::post(full_url)
        .send_form(body)?
        .body_mut()
        .read_to_string()?;
    println!("get_oauth_url{}", url);
    Ok(url)
}


fn wait_for_redirect() -> u16 {
    let mut port = 53400;
    let mut server = None;
    for _ in 0..MAX_RETRIES { 

        let url = format!("0.0.0.0:{}", port);
        let result = rouille::Server::new(&url, move |request| {
            router!(request,
                (POST) (/callback) => {
                    let data = try_or_400!(post_input!(request, {
                        access_token: String,
                        refresh_token: String,
                        expires_in: String,
                    }));

                    println!("Got callback with data: {:?}", data);

                    let html_content = match std::fs::read_to_string("success.html") {
                        Ok(content) => content,
                        Err(_) => return rouille::Response::text("Failed to read template") 
                    };

                    rouille::Response::html(html_content)
                },

                _ => {
                    println!("Got request for unknown path");
                    rouille::Response::empty_404()
                }
            )
        });

        match result {
            Ok(_server) => {
                println!("Server started successfully on {}", url);
                server = Some((_server, port));
                break;
            }

            Err(err) => {
                eprintln!("Failed to start server on {}: {}", url, err);
                port += 1;
                println!("Retrying with port {}", port);
            }
        }
    }


    if let Some((_, port)) = server {
        return port;
    } else {
        return 0;
    }

}

