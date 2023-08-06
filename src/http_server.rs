use std::env;

use axum::response::IntoResponse;
use axum::routing::get;
use axum::*;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{event, Level};

use crate::graphql_server::run_graphql_server;
use hyper::StatusCode;
use tokio::signal;

pub(crate) async fn run_http_server() {
    event!(Level::INFO, "Starting HTTP server");
    let media_root_dir = shellexpand::env(
        &env::var("MEDIA_ROOT").expect("'MEDIA_ROOT' environment variable is required"),
    )
    .unwrap()
    .to_string();

    let app = Router::new()
        .nest_service("/media", ServeDir::new(media_root_dir))
        .route("/healthz", get(liveness_handler))
        .route("/readyz", get(ready_handler));

    Server::bind(&"0.0.0.0:8998".parse().unwrap())
        .serve(
            run_graphql_server(app)
                .await
                .layer(CorsLayer::permissive())
                .layer(TraceLayer::new_for_http())
                .into_make_service(),
        )
        // .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

#[allow(dead_code)]
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

    event!(Level::INFO, "signal received, starting graceful shutdown");
}

async fn ready_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn liveness_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
