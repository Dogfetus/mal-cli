#![allow(unreachable_code)]
use std::sync::mpsc::{self};
use std::time::Duration;
use rouille::try_or_400;
use rouille::post_input;
use rouille::router;
use anyhow::Result;
use std::thread;
use open;
use ureq;



const MAX_RETRIES: u16 = 10;
const BACKEND_URL: &str = "https://mal-cli.dogfetus.no";



pub fn oauth_login<F>(callback: F) 
where
    F: FnOnce(String, String, String) -> Result<()> + Send + Sync + 'static + Copy
{
    if let Some((port, joinable)) = start_callback_server(callback) {
        let _url = get_oauth_url(port).expect("Can't connect to backend");
        joinable.join().unwrap();
    } 
}


fn get_oauth_url(port: u16) -> Result<String> {
    let full_url = format!("{}/oauth_url", BACKEND_URL); 
    let body = [ ("port", port.to_string()) ];
    let url: String = ureq::post(full_url)
        .send_form(body)?
        .body_mut()
        .read_to_string()?;
    open::that(&url)
        .expect("Failed to open browser");
    println!("if the browser didn't open, visit this url: {}", url);
    Ok(url)
}



/* 
* This function starts a local server to listen for the callback from the OAuth provider.
* it takes a callback function as an argument, which will be called when the server receives a callback.
* The callback function should accept three parameters: access_token, refresh_token, and expires_in.
* It will return the port number on which the server is running.
* */
fn start_callback_server<F>(callback: F) -> Option<(u16, thread::JoinHandle<()>)> 
where
    F: FnOnce(String, String, String) -> Result<()> + Send + Sync + 'static + Copy
{
    let mut port: u16 = 53400;
    let (tx, rx) = mpsc::channel::<()>();

    for _ in 0..MAX_RETRIES { 
        let _tx = tx.clone();
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

                    let html_content = match std::fs::read_to_string("src/templates/success.html") {
                        Ok(content) => content,
                        Err(_) => return rouille::Response::text("Failed to read template") 
                    };

                    match callback(data.access_token, data.refresh_token, data.expires_in) {
                        Ok(_) => {}, 
                        Err(err) => {
                            return rouille::Response::text(format!("Callback failed, error: {}", err));
                        },
                    }

                    let _ = _tx.send(()); 
                    rouille::Response::html(html_content)
                },

                _ => {
                    println!("Got request for unknown path");
                    rouille::Response::empty_404()
                }
            )
        });

        match result {
            Ok(server) => {
                println!("Server started on port {}", port);
                let (handle, sender) = server.stoppable();
                let joinable = thread::spawn(move || {
                    let _ = rx.recv();
                    println!("Stopping server on {}", url);

                    thread::sleep(Duration::from_secs(1));
                    sender.send(()).unwrap();
                    handle.join().unwrap();
                    println!("Server stopped");
                });

                return Some((port, joinable));
            }

            Err(err) => {
                eprintln!("Failed to start server on {}: {}", url, err);
                port += 1;
                println!("Retrying with port {}", port);
            }
        }
    }


    println!("Failed to start server after {} retries", MAX_RETRIES);
    None
}

