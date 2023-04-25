pub mod models;
pub mod routes;
mod utils;

use axum::{http::StatusCode, response::IntoResponse, Json};
use sea_orm::TransactionError;
use std::error::Error as StdError;
// use tracing::debug;

use crate::ErrorHandlingResponse;

#[derive(thiserror::Error, Debug)]
pub enum ChaptersError {
    #[error("internal server error")]
    InternalServerError,

    #[error("chapter not found")]
    ChapterNotFound,

    #[error("bad request")]
    BadRequest,

    #[error("image size too large, maximum image size is 5MB")]
    ImageTooLarge,

    #[error("jwt internal server error")]
    JWT(#[from] jwt_simple::Error),

    #[error("sea_orm internal server error")]
    SeaORM(#[from] sea_orm::error::DbErr),

    // #[error("validation error: {0}")]
    // Validator(#[from] validator::ValidationErrors),
    #[error("{0}")]
    Conflict(String),
}

impl<E> From<TransactionError<E>> for ChaptersError
where
    E: StdError + Into<ChaptersError>,
{
    fn from(err: TransactionError<E>) -> Self {
        match err {
            TransactionError::Connection(db) => Self::SeaORM(db),
            TransactionError::Transaction(err) => err.into(),
        }
    }
}

impl IntoResponse for ChaptersError {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("{}", self.to_string());

        let (status, error_message) = match self {
            ChaptersError::ChapterNotFound => (StatusCode::NOT_FOUND, vec![self.to_string()]),
            ChaptersError::BadRequest => (StatusCode::BAD_REQUEST, vec![self.to_string()]),
            ChaptersError::ImageTooLarge => (StatusCode::BAD_REQUEST, vec![self.to_string()]),
            ChaptersError::Conflict(_) => (StatusCode::CONFLICT, vec![self.to_string()]),
            ChaptersError::SeaORM(_) => (StatusCode::INTERNAL_SERVER_ERROR, vec![self.to_string()]),
            ChaptersError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, vec![self.to_string()])
            }
            ChaptersError::JWT(_) => {
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
