use std::sync::LazyLock;

use axum::Json;
use axum::http::{HeaderName, StatusCode};

pub const HEADER_X_APP_TOKEN: HeaderName = HeaderName::from_static("x-app-token");

pub static RESPONSE_ERROR_FORBIDDEN: LazyLock<(StatusCode, Json<serde_json::Value>)> =
    LazyLock::new(|| (StatusCode::FORBIDDEN, Json(serde_json::json!({"message": "Forbidden"}))));
pub static RESPONSE_ERROR_INTERNAL_SERVER_ERROR: LazyLock<(StatusCode, Json<serde_json::Value>)> =
    LazyLock::new(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message": "Internal Server Error"})),
        )
    });
pub static RESPONSE_ERROR_UNAUTHORIZED: LazyLock<(StatusCode, Json<serde_json::Value>)> = LazyLock::new(|| {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"message": "Unauthorized"})),
    )
});
