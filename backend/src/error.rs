use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Unified application error type. Every fallible handler returns
/// `Result<T, AppError>`; this converts cleanly into an HTTP JSON response
/// so a failure in Neo4j or MusicBrainz never panics a request handler.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("invalid request: {0}")]
    BadRequest(String),

    #[error("database error: {0}")]
    Database(#[from] neo4rs::Error),

    #[error("MusicBrainz request failed: {0}")]
    MusicBrainz(#[from] reqwest::Error),

    #[error("upstream MusicBrainz error: {0}")]
    MusicBrainzApi(String),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Database(e) => {
                tracing::error!(error = %e, "neo4j error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database error".to_string(),
                )
            }
            AppError::MusicBrainz(e) => {
                tracing::error!(error = %e, "musicbrainz request error");
                (
                    StatusCode::BAD_GATEWAY,
                    "failed to reach MusicBrainz".to_string(),
                )
            }
            AppError::MusicBrainzApi(msg) => {
                tracing::error!(error = %msg, "musicbrainz api error");
                (StatusCode::BAD_GATEWAY, msg.clone())
            }
            AppError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
