mod file_management;
mod graphql_server;
mod http_server;
mod model;
use std::env;

use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }
    http_server::run_http_server().await;
}
