use crate::model::{new_schema, ServiceSchema};
// use async_graphql::*;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig, ALL_WEBSOCKET_PROTOCOLS};
// use async_graphql_axum::*;
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
// use axum::debug_handler;
use axum::{
    extract::{Extension, WebSocketUpgrade},
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/ws"),
    ))
}

// #[debug_handler]
async fn graphql_handler(
    schema: Extension<ServiceSchema>,
    _headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    // println!("GQL request. Headers: {:?}", headers);
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
            GraphQLWebSocket::new(stream, schema, protocol)
                // for adding token-from-header support, see https://github.com/async-graphql/examples/tree/master/models/token
                // .on_connection_init(on_connection_init)
                .serve()
        })
}

pub(crate) async fn run_graphql_server(router: Router) -> Router {
    // async-graphql-examples
    // https://github.com/async-graphql/examples
    println!(
        "Photomanager graphql server.\nVisit http://localhost:8998/graphql to use the playground."
    );
    router
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .route("/ws", get(graphql_ws_handler))
        .layer(Extension(new_schema()))
}
