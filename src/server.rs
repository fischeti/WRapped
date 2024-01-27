use std::thread;
use std::time::Duration;

use actix_files as fs;
use actix_web::{App, HttpServer};

use log::{error, info};

pub fn open_browser(server: &str) {
    thread::sleep(Duration::from_secs(1));
    if let Err(e) = open::that(server) {
        error!("Failed to open browser: {}", e);
    }
}

pub async fn run_server(server: &str) -> std::io::Result<()> {
    info!("Starting the web server at {}", server);
    HttpServer::new(move || {
        App::new()
            // Your routes and server configuration here
            .service(fs::Files::new("/stats", "./shared"))
            .service(fs::Files::new("/", "./web/").index_file("index.html"))
    })
    .workers(1)
    .bind(server)?
    .run()
    .await
}
