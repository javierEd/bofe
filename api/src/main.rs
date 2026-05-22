use std::net::SocketAddr;

use async_graphql::extensions::apollo_persisted_queries::{ApolloPersistedQueries, LruCacheStorage};
use async_graphql::extensions::{ApolloTracing, Logger};
use axum::Router;
use axum::http::Method;
use axum::routing::{get, post};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use bofe_core::graphql::{GraphqlSchema, GraphqlSchemaExt};
use bofe_core::start_tracing_subscriber;

use crate::config::API_CONFIG;
use crate::handlers::{get_graphql_ws, get_index, post_graphql};

mod config;
mod constants;
mod handlers;

#[tokio::main]
async fn main() {
    start_tracing_subscriber();

    let mut graphql_schema_builder = GraphqlSchema::builder()
        .extension(ApolloPersistedQueries::new(LruCacheStorage::new(1024)))
        .extension(Logger);

    graphql_schema_builder = if !cfg!(debug_assertions) {
        graphql_schema_builder.disable_introspection()
    } else {
        graphql_schema_builder.extension(ApolloTracing)
    };

    let graphql_schema = graphql_schema_builder.finish();

    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let router = Router::new()
        .route("/", get(get_index))
        .route("/graphql", post(post_graphql))
        .route("/ws", get(get_graphql_ws))
        .with_state(graphql_schema)
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer)
        .layer(API_CONFIG.client_ip_source.clone().into_extension());

    let api_address = &API_CONFIG.address;

    let listener = TcpListener::bind(&api_address).await.unwrap();

    tracing::info!("Listening on http://{api_address}");

    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
