mod file_management;
mod graphql_server;
mod http_server;
mod model;

#[tokio::main]
async fn main() {
    http_server::run_http_server().await;
}
