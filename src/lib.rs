mod file_management;
pub mod fsops;
mod google_photos_upload;
mod graphql_server;
mod http_server;
mod image;
pub mod model;
mod reqwops;
pub mod reviewscore;
use dotenvy::dotenv;

pub async fn run_server() {
    dotenv().ok();
    console_subscriber::init();
    if let Err(e) = http_server::run_http_server().await {
        tracing::error!("Failed to run the HTTP server: {e}");
    }
}
