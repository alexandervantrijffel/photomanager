use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
#[derive(Serialize)]
struct Health {
    healthy: bool,
}

pub(crate) async fn health() -> impl IntoResponse {
    let health = Health { healthy: true };
    (StatusCode::OK, Json(health))
}
