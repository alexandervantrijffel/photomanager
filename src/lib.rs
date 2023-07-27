mod file_management;
pub mod fsops;
mod graphql_server;
mod http_server;
mod image;
pub mod model;
pub mod reviewscore;

use dotenv::dotenv;

pub async fn run_server() {
    dotenv().ok();
    http_server::run_http_server().await;
}
