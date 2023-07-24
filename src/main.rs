mod file_management;
mod graphql_server;
mod http_server;
mod image;
mod model;

use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    http_server::run_http_server().await;
}
