use axum::response::IntoResponse;
use axum::routing::get;
use axum::*;
use tower_http::cors::{Any, CorsLayer};

use crate::graphql_server::run_graphql_server;
use hyper::{Method, StatusCode};
use serde::Serialize;
use tokio::signal;

pub(crate) async fn run_http_server() {
    let app = Router::new();
    let app = run_graphql_server(app).await;
    let app = app.route("/health", get(health_handler)).layer(
        CorsLayer::new().allow_origin(Any).allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::DELETE,
        ]),
    );
    Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

#[derive(Serialize)]
struct Health {
    healthy: bool,
}

async fn health_handler() -> impl IntoResponse {
    let health = Health { healthy: true };
    (StatusCode::OK, Json(health))
}
