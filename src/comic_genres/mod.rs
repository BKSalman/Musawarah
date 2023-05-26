use axum::{http::StatusCode, response::IntoResponse};
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

        match self {
            ComicGenresError::PlaceHolder => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            ComicGenresError::InvalidGenre => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            ComicGenresError::PoolError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            ComicGenresError::Diesel(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}
