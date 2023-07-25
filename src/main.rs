mod file_management;
mod fsops;
mod graphql_server;
mod http_server;
mod image;
mod model;
mod reviewscore;

use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    http_server::run_http_server().await;
}
