use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug)]
pub struct SimpleError(pub String);

impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SimpleError {}
#[derive(Debug, Error)]
pub enum KestrelError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Bad gateway: {0}")]
    BadGateway(String),
    #[error("Service unavailable: {0}")]
    Unavailable(String),
    #[error("Re-authentication required: {0}")]
    ReauthRequired(String),
    #[error("Internal error: {0}")]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl IntoResponse for KestrelError {
    fn into_response(self) -> Response {
        let status = match &self {
            KestrelError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            KestrelError::NotFound(_) => StatusCode::NOT_FOUND,
            KestrelError::Unauthorized => StatusCode::UNAUTHORIZED,
            KestrelError::ReauthRequired(_) => StatusCode::UNAUTHORIZED,
            KestrelError::Forbidden(_) => StatusCode::FORBIDDEN,
            KestrelError::Conflict(_) => StatusCode::CONFLICT,
            KestrelError::BadRequest(_) => StatusCode::BAD_REQUEST,
            KestrelError::BadGateway(_) => StatusCode::BAD_GATEWAY,
            KestrelError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            KestrelError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = match &self {
            KestrelError::ReauthRequired(msg) => axum::Json(serde_json::json!({
                "error": msg,
                "reauth_required": true,
            })),
            _ => axum::Json(serde_json::json!({
                "error": self.to_string(),
            })),
        };

        (status, body).into_response()
    }
}
