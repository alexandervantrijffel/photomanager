mod file_management;
mod graphql_server;
mod model;

use graphql_server::run_graphql_server;

#[tokio::main]
async fn main() {
    run_graphql_server().await;
}
