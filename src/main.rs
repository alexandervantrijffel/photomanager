mod model;
mod routes;
use crate::routes::health;
use axum::{routing::get, Router, Server};
#[tokio::main]
async fn main() {
    let app = Router::new().route("/health", get(health));
    println!("Starting tokio server");
    Server::bind(&"0.0.0.0:8000".parse().unwrap()) // (3)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
