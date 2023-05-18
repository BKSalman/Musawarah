use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;
// use tracing::debug;

use crate::ErrorResponse;

pub mod models;
pub mod routes;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ComicsParams {
    #[serde(default = "Uuid::nil")]
    min_id: Uuid,
    #[serde(default = "Uuid::max")]
    max_id: Uuid,
    #[serde(default)]
    genre: Option<i32>,
}

#[derive(thiserror::Error, Debug)]
pub enum ComicsError {
    #[error("internal server error")]
    InternalServerError,

    #[error("comic not found")]
    ComicNotFound,

    #[error("bad request")]
    BadRequest,

    #[error("image size too large, maximum image size is 10MB")]
    ImageTooLarge,

    #[error("internal server error")]
    SeaORM(#[from] sea_orm::DbErr),

    // #[error("validation error: {0}")]
    // Validator(#[from] validator::ValidationErrors),
    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    ComicGenresErrors(#[from] crate::comic_genres::ComicGenresError),
}

impl IntoResponse for ComicsError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        let (status, error_message) = match self {
            ComicsError::ComicNotFound => (StatusCode::NOT_FOUND, vec![self.to_string()]),
            ComicsError::BadRequest => (StatusCode::BAD_REQUEST, vec![self.to_string()]),
            ComicsError::ImageTooLarge => (StatusCode::BAD_REQUEST, vec![self.to_string()]),
            ComicsError::Conflict(_) => (StatusCode::CONFLICT, vec![self.to_string()]),
            ComicsError::SeaORM(_) => (StatusCode::INTERNAL_SERVER_ERROR, vec![self.to_string()]),
            ComicsError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, vec![self.to_string()])
            }
            ComicsError::ComicGenresErrors(_) => (StatusCode::BAD_REQUEST, vec![self.to_string()]),
        };

        let body = Json(ErrorResponse {
            errors: error_message,
        });

        (status, body).into_response()
    }
}
