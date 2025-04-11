#![allow(unreachable_code)]
use ureq;
use rouille::router;
use anyhow::Result;
use std::thread;
use std::time::Duration;
use rouille::try_or_400;
use rouille::post_input;



static BACKEND_URL: &str = "https://mal-cli.dogfetus.no";


//TODO: SEND PORT TO BACKEND 
pub async fn oauth_login() {
    let url = get_oauth_url().unwrap();
    let port = wait_for_redirect();
    println!("url: {}", url);
}


fn get_oauth_url() -> Result<String> {
    let full_url = format!("{}/oauth_url", BACKEND_URL); 
    let body = [ ("port", "6969") ];
    let url: String = ureq::post(full_url)
        .send_form(body)?
        .body_mut()
        .read_to_string()?;
    println!("get_oauth_url{}", url);
    Ok(url)
}


fn wait_for_redirect() {
    let url = format!("0.0.0.0:{}", 6969);
    println!("Now listening on {}", url);

    let result = rouille::Server::new(&url, move |request| {
        router!(request,
            (POST) (/callback) => {
                let data = try_or_400!(post_input!(request, {
                    access_token: String,
                    refresh_token: String,
                    expires_in: String,
                }));

                println!("Got callback with data: {:?}", data);

                rouille::Response::html(r#"
                    <html>
                        <head>
                            <title>Success</title>
                        </head>
                        <body style="font-family: sans-serif; text-align: center; padding-top: 50px;">
                            <h1>✅ Login successful!</h1>
                            <p>You may now close this window and return to the terminal.</p>
                        </body>
                    </html>
                "#)
            },

            _ => {
                println!("Got request for unknown path");
                rouille::Response::empty_404()
            }
        )
    });

    let server = match result {
        Ok(server) => server,
        Err(err) => {
            eprintln!("Failed to start server on port {}: {}", url, err);
            return; // or handle it however you like
        }
    };

    let (handle, sender) = server.stoppable();

    // Stop the server in 3 seconds
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(100));
        sender.send(()).unwrap();
    });

    // Block the main thread until the server is stopped
    handle.join().unwrap();
}

