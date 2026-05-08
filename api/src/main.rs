use std::net::SocketAddr;

use async_graphql::extensions::apollo_persisted_queries::{ApolloPersistedQueries, LruCacheStorage};
use async_graphql::extensions::{ApolloTracing, Logger};
use async_graphql_axum::{GraphQLBatchRequest, GraphQLResponse};
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, Request};
use axum::response::{IntoResponse, Result};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_client_ip::ClientIp;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use sentry::integrations::tower::{NewSentryLayer, SentryHttpLayer};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use toolbox::constants::RESPONSE_ERROR_UNAUTHORIZED;
use toolbox::tracing::start_tracing_subscriber;

use boards_core::Info;
use boards_core::graphql::{GraphqlSchema, GraphqlSchemaExt};

use crate::config::API_CONFIG;

mod config;

async fn get_index() -> impl IntoResponse {
    Json(Info::default())
}

async fn post_graphql(
    State(schema): State<GraphqlSchema>,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    ClientIp(client_ip): ClientIp,
    batch_request: GraphQLBatchRequest,
) -> Result<GraphQLResponse> {
    let Some(TypedHeader(Authorization(bearer))) = authorization else {
        return Err(RESPONSE_ERROR_UNAUTHORIZED.clone().into());
    };

    let _token = bearer.token().to_owned();

    let batch_request = batch_request.into_inner().data(client_ip);

    Ok(schema.execute_batch(batch_request).await.into())
}

#[tokio::main]
async fn main() {
    let _guard = start_tracing_subscriber();

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
        .with_state(graphql_schema)
        .layer(SentryHttpLayer::new().enable_transaction())
        .layer(NewSentryLayer::<Request<Body>>::new_from_top())
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
