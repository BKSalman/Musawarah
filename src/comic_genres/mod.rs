use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::ErrorResponse;

pub mod models;
pub mod routes;

#[derive(Debug, thiserror::Error)]
pub enum ComicGenresError {
    #[error("salman forgot to handle this properly")]
    PlaceHolder,

    #[error("invalid genre")]
    InvalidGenre,
}

impl IntoResponse for ComicGenresError {
    fn into_response(self) -> axum::response::Response {
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
        };

        let body = Json(error_message);

        (error_status, body).into_response()
    }
}
