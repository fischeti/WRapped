use actix_files as fs;
use actix_web::{App, HttpServer};

use log::{info, error};

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
