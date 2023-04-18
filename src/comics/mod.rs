use axum::{http::StatusCode, response::IntoResponse, Json};
// use tracing::debug;

use crate::ErrorHandlingResponse;

pub mod models;
pub mod routes;

#[derive(thiserror::Error, Debug)]
pub enum ComicsError {
    #[error("internal server error")]
    InternalServerError,

    #[error("post not found")]
    PostNotFound,

    #[error("bad request")]
    BadRequest,

    #[error("image size too large, maximum image size is 5MB")]
    ImageTooLarge,

    #[error("jwt internal server error")]
    JWT(#[from] jwt_simple::Error),

    #[error("sqlx internal server error")]
    Sqlx(#[from] sqlx::Error),

    // #[error("validation error: {0}")]
    // Validator(#[from] validator::ValidationErrors),
    #[error("{0}")]
    Conflict(String),
}

impl IntoResponse for ComicsError {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("{}", self.to_string());

        let (status, error_message) = match self {
            ComicsError::PostNotFound => (StatusCode::NOT_FOUND, vec![self.to_string()]),
            ComicsError::BadRequest => (StatusCode::BAD_REQUEST, vec![self.to_string()]),
            ComicsError::ImageTooLarge => (StatusCode::BAD_REQUEST, vec![self.to_string()]),
            ComicsError::Conflict(_) => (StatusCode::CONFLICT, vec![self.to_string()]),
            ComicsError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR, vec![self.to_string()]),
            ComicsError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, vec![self.to_string()])
            }
            ComicsError::JWT(_) => {
                // TODO: add logging for this
                (StatusCode::INTERNAL_SERVER_ERROR, vec![self.to_string()])
            }
        };

        let body = Json(ErrorHandlingResponse {
            errors: error_message,
        });

        (status, body).into_response()
    }
}
