use async_graphql::Data;
use async_graphql::http::ALL_WEBSOCKET_PROTOCOLS;
use async_graphql_axum::{GraphQLBatchRequest, GraphQLProtocol, GraphQLResponse, GraphQLWebSocket};
use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use axum::response::{IntoResponse, Result};
use axum_client_ip::ClientIp;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;

use bofe_core::graphql::GraphqlSchema;
use bofe_core::{Info, commands};
use serde::Deserialize;
use uuid::Uuid;

use crate::constants::*;

#[derive(Deserialize)]
pub struct AvatarImageQuery {
    pub size: Option<u16>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct WsInitialPayload {
    app_token: String,
    session_token: Option<String>,
}

trait OrHttpError<T> {
    #[allow(clippy::result_large_err, dead_code)]
    fn or_forbidden(self) -> Result<T>;

    fn or_not_found(self) -> Result<T>;

    #[allow(clippy::result_large_err)]
    fn or_internal_server_error(self) -> Result<T>;

    #[allow(clippy::result_large_err)]
    fn or_unauthorized(self) -> Result<T>;
}

impl<T> OrHttpError<T> for Option<T> {
    fn or_forbidden(self) -> Result<T> {
        self.ok_or_else(|| RESPONSE_ERROR_FORBIDDEN.clone().into())
    }

    fn or_not_found(self) -> Result<T> {
        self.ok_or_else(|| RESPONSE_ERROR_NOT_FOUND.clone().into())
    }

    fn or_internal_server_error(self) -> Result<T> {
        self.ok_or_else(|| RESPONSE_ERROR_INTERNAL_SERVER_ERROR.clone().into())
    }

    fn or_unauthorized(self) -> Result<T> {
        self.ok_or_else(|| RESPONSE_ERROR_UNAUTHORIZED.clone().into())
    }
}

impl<T, E> OrHttpError<T> for Result<T, E> {
    fn or_forbidden(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(_) => Err(RESPONSE_ERROR_FORBIDDEN.clone().into()),
        }
    }

    fn or_not_found(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(_) => Err(RESPONSE_ERROR_NOT_FOUND.clone().into()),
        }
    }

    fn or_internal_server_error(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(_) => Err(RESPONSE_ERROR_INTERNAL_SERVER_ERROR.clone().into()),
        }
    }

    fn or_unauthorized(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(_) => Err(RESPONSE_ERROR_UNAUTHORIZED.clone().into()),
        }
    }
}

pub async fn get_graphql_ws(
    State(schema): State<GraphqlSchema>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> Result<impl IntoResponse> {
    Ok(websocket.protocols(ALL_WEBSOCKET_PROTOCOLS).on_upgrade(move |stream| {
        GraphQLWebSocket::new(stream, schema.clone(), protocol)
            .on_connection_init(move |value: serde_json::Value| async move {
                let Ok(payload) = serde_json::from_value::<WsInitialPayload>(value) else {
                    return Err("Could not parse initial payload".into());
                };

                let mut data = Data::default();

                let application = commands::get_application_by_token(&payload.app_token).await?;

                data.insert(application);

                if let Some(session_token) = payload.session_token {
                    let session = commands::get_session_by_token(&session_token).await?;
                    let user = session.user().await?;

                    data.insert(session);
                    data.insert(user);
                }

                Ok(data)
            })
            .serve()
    }))
}

pub async fn get_index() -> impl IntoResponse {
    Json(Info::default())
}

pub async fn get_user_avatar_image(
    Path(id): Path<Uuid>,
    Query(params): Query<AvatarImageQuery>,
) -> Result<impl IntoResponse> {
    let size = params.size.unwrap_or(256);

    if size < 16 || size > 512 || size & (size - 1) != 0 {
        return Err(RESPONSE_ERROR_BAD_REQUEST.clone().into());
    }

    let user = commands::get_user_by_id(id).await.or_not_found()?;

    let avatar_image = user.avatar_image(size).or_internal_server_error()?;

    let content_length = avatar_image.len();
    let body = Body::from(avatar_image);

    let headers = [
        (CONTENT_TYPE, "image/jpeg".to_owned()),
        (CONTENT_LENGTH, content_length.to_string()),
        (
            CONTENT_DISPOSITION,
            format!("inline; filename=\"{}_{}x{}.jpg\"", id, size, size),
        ),
    ];

    Ok((headers, body))
}

pub async fn post_graphql(
    headers: HeaderMap,
    State(schema): State<GraphqlSchema>,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    ClientIp(client_ip): ClientIp,
    batch_request: GraphQLBatchRequest,
) -> Result<GraphQLResponse> {
    let app_token = headers
        .get(HEADER_X_APP_TOKEN)
        .or_forbidden()?
        .to_str()
        .or_forbidden()?;

    let application = commands::get_application_by_token(app_token).await.or_forbidden()?;

    let mut batch_request = batch_request.into_inner().data(client_ip).data(application);

    if let Some(TypedHeader(Authorization(bearer))) = authorization {
        let token = bearer.token().to_owned();

        let session = commands::get_session_by_token(&token).await.or_unauthorized()?;
        let user = session.user().await.or_internal_server_error()?;

        batch_request = batch_request.data(session).data(user);
    }

    Ok(schema.execute_batch(batch_request).await.into())
}
