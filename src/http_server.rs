use crate::graphql_server::run_graphql_server;
use axum::Router;
use axum::response::IntoResponse;
use axum::routing::get;
// only import this as dev-dependency
// #[cfg(debug_assertions)]
use listenfd::ListenFd;
use std::env;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info;

pub(crate) async fn run_http_server() -> anyhow::Result<()> {
    info!("Starting HTTP server");
    let media_root_dir: String = shellexpand::env(
        &env::var("MEDIA_ROOT").expect("'MEDIA_ROOT' environment variable is required"),
    )
    .unwrap()
    .into();

    let app = Router::new()
        .nest_service("/media", ServeDir::new(media_root_dir))
        .route("/healthz", get(liveness_handler))
        .route("/readyz", get(ready_handler));

    let app = app
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let app = run_graphql_server(app)
        .await
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // listenfd keeps a socket open on the target port,
    // so that a request from the browser does not fail during compilation
    let mut listenfd = ListenFd::from_env();
    let listen_addr = "0.0.0.0:8998";

    let listener = if let Some(listener) = listenfd.take_tcp_listener(0)? {
        info!("Using listener from listenfd");
        let _ = listener.set_nonblocking(true);
        tokio::net::TcpListener::from_std(listener)?
    } else {
        info!("Using new TcpListener on {}", listen_addr);
        tokio::net::TcpListener::bind(listen_addr).await?
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
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
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}

async fn ready_handler() -> impl IntoResponse {
    (axum::http::StatusCode::OK, "OK")
}

async fn liveness_handler() -> impl IntoResponse {
    (axum::http::StatusCode::OK, "OK")
}
