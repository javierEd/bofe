use async_graphql_axum::{GraphQLBatchRequest, GraphQLResponse};
use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Result};

use axum_client_ip::ClientIp;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;

use bofe_core::graphql::GraphqlSchema;
use bofe_core::{Info, commands};

use crate::constants::*;

trait OrHttpError<T> {
    #[allow(clippy::result_large_err, dead_code)]
    fn or_forbidden(self) -> Result<T>;

    #[allow(clippy::result_large_err)]
    fn or_internal_server_error(self) -> Result<T>;

    #[allow(clippy::result_large_err)]
    fn or_unauthorized(self) -> Result<T>;
}

impl<T> OrHttpError<T> for Option<T> {
    fn or_forbidden(self) -> Result<T> {
        self.ok_or_else(|| RESPONSE_ERROR_FORBIDDEN.clone().into())
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

pub async fn get_index() -> impl IntoResponse {
    Json(Info::default())
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
