use std::thread;
use std::time::Duration;

use actix_web::http::header::ContentType;
use actix_web::{App, HttpResponse, HttpServer, web};
use rust_embed::RustEmbed;

use log::{error, info};

/// The web app, baked into the binary at compile time so that WRapped can be
/// run from anywhere instead of only from a checkout.
///
/// In debug builds `rust-embed` reads the files from disk on every request
/// instead, so that the app can still be edited without recompiling.
#[derive(RustEmbed)]
#[folder = "web/"]
struct Web;

/// Serve one of the embedded files, guessing its content type from the
/// extension.
fn serve(path: &str) -> HttpResponse {
    match Web::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime.as_ref())
                .body(file.data.into_owned())
        }
        None => HttpResponse::NotFound()
            .content_type(ContentType::plaintext())
            .body(format!("No such file: {}", path)),
    }
}

async fn index() -> HttpResponse {
    serve("index.html")
}

async fn asset(path: web::Path<String>) -> HttpResponse {
    serve(&path)
}

/// Serve the statistics of this run, which are kept in memory instead of
/// being written next to the binary.
async fn stats(stats: web::Data<String>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(stats.get_ref().clone())
}

pub fn open_browser(server: &str) {
    thread::sleep(Duration::from_secs(1));
    if let Err(e) = open::that(server) {
        error!("Failed to open browser: {}", e);
    }
}

/// The routes of the web app, so that the tests can serve them without
/// binding a port.
fn routes(app: &mut web::ServiceConfig) {
    app
        // Registered before the catch-all, which would shadow it otherwise
        .route("/stats/stats.json", web::get().to(stats))
        .route("/", web::get().to(index))
        .route("/{path:.*}", web::get().to(asset));
}

pub async fn run_server(server: &str, json: String) -> std::io::Result<()> {
    info!("Starting the web server at {}", server);
    let json = web::Data::new(json);
    HttpServer::new(move || App::new().app_data(json.clone()).configure(routes))
        .workers(1)
        .bind(server)?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_the_whole_web_app() {
        for file in ["index.html", "css/styles.css", "js/main.js"] {
            assert!(Web::get(file).is_some(), "`{}` is not embedded", file);
        }
    }

    #[test]
    fn embeds_d3_instead_of_loading_it_from_a_cdn() {
        assert!(Web::get("js/d3.v7.min.js").is_some());

        let index = Web::get("index.html").expect("index.html is not embedded");
        let index = String::from_utf8_lossy(&index.data);
        assert!(
            !index.contains("d3js.org"),
            "index.html still loads D3 from the CDN"
        );
    }

    #[test]
    fn guesses_the_content_type_of_an_asset() {
        assert_eq!(
            serve("css/styles.css")
                .headers()
                .get("content-type")
                .unwrap(),
            "text/css"
        );
        assert_eq!(
            serve("js/main.js").headers().get("content-type").unwrap(),
            "text/javascript"
        );
    }

    #[test]
    fn reports_a_missing_asset_instead_of_panicking() {
        assert_eq!(
            serve("js/nonexistent.js").status(),
            actix_web::http::StatusCode::NOT_FOUND
        );
    }

    /// Serve the app with `json` as the statistics, and answer one `GET`.
    async fn get(path: &str, json: &str) -> (actix_web::http::StatusCode, String) {
        let app = actix_web::test::init_service(
            App::new()
                .app_data(web::Data::new(json.to_string()))
                .configure(routes),
        )
        .await;

        let request = actix_web::test::TestRequest::get().uri(path).to_request();
        let response = actix_web::test::call_service(&app, request).await;
        let status = response.status();
        let body = actix_web::test::read_body(response).await;

        (status, String::from_utf8_lossy(&body).into_owned())
    }

    #[actix_web::test]
    async fn serves_the_index_at_the_root() {
        let (status, body) = get("/", "{}").await;

        assert!(status.is_success());
        assert!(body.contains("<title>"), "the root is not the index page");
    }

    #[actix_web::test]
    async fn serves_the_statistics_the_web_app_fetches() {
        // The path `main.js` fetches, which the catch-all must not swallow
        let (status, body) = get("/stats/stats.json", r#"{"year":2023}"#).await;

        assert!(status.is_success());
        assert_eq!(body, r#"{"year":2023}"#);
    }

    #[actix_web::test]
    async fn serves_the_assets_the_index_links_to() {
        for path in ["/css/styles.css", "/js/main.js", "/js/d3.v7.min.js"] {
            let (status, body) = get(path, "{}").await;

            assert!(status.is_success(), "could not serve `{}`", path);
            assert!(!body.is_empty(), "`{}` was served empty", path);
        }
    }

    #[actix_web::test]
    async fn answers_an_unknown_path_with_a_404() {
        let (status, _) = get("/nonexistent", "{}").await;

        assert_eq!(status, actix_web::http::StatusCode::NOT_FOUND);
    }
}
