use crate::model::QueryRoot;
use crate::model::ServiceSchema;
// use async_graphql::*;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig, ALL_WEBSOCKET_PROTOCOLS};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
// use async_graphql_axum::*;
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{Extension, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router, Server,
};
// use axum_macros::debug_handler;
use hyper::Method;
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize)]
struct Health {
    healthy: bool,
}

async fn health_handler() -> impl IntoResponse {
    let health = Health { healthy: true };
    (StatusCode::OK, Json(health))
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/ws"),
    ))
}

// #[debug_handler]
async fn graphql_handler(
    schema: Extension<ServiceSchema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    println!("GQL request. Headers: {:?}", headers);
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_ws_handler(
    Extension(schema): Extension<ServiceSchema>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> Response {
    websocket
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema.clone(), protocol)
                // for adding token-from-header support, see https://github.com/async-graphql/examples/tree/master/models/token
                // .on_connection_init(on_connection_init)
                .serve()
        })
}

pub(crate) async fn run_graphql_server() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
    // async-graphql-examples
    // https://github.com/async-graphql/examples
    println!("Running photomanager graphql server. Visit http://localhost:8000/graphql to use the playground.");
    let app = Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .route("/ws", get(graphql_ws_handler))
        .route("/health", get(health_handler))
        .layer(Extension(schema))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::DELETE,
        ]));
    Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
