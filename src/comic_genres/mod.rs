use axum::{http::StatusCode, response::IntoResponse, Json};
use diesel_async::pooled_connection::deadpool::PoolError;

use crate::ErrorResponse;

pub mod models;
pub mod routes;

#[derive(Debug, thiserror::Error)]
pub enum ComicGenresError {
    #[error("salman forgot to handle this properly")]
    PlaceHolder,

    #[error("internal server error")]
    Diesel(#[from] diesel::result::Error),

    #[error("internal server error")]
    PoolError(#[from] PoolError),

    #[error("invalid genre")]
    InvalidGenre,
}

impl IntoResponse for ComicGenresError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        let (error_status, error_message) = match self {
            ComicGenresError::PlaceHolder => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            ComicGenresError::InvalidGenre => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            ComicGenresError::PoolError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            ComicGenresError::Diesel(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
        };

        let body = Json(error_message);

        (error_status, body).into_response()
    }
}
