pub mod models;
pub mod routes;
mod utils;

use axum::{http::StatusCode, response::IntoResponse, Json};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::{ErrorResponse, SortingOrder};
// use tracing::debug;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ChaptersParams {
    #[serde(default = "Uuid::nil")]
    pub min_id: Uuid,
    #[serde(default = "Uuid::max")]
    pub max_id: Uuid,
    #[serde(default)]
    pub sorting: Option<SortingOrder>,
}

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

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error(transparent)]
    Validator(#[from] garde::Errors),

    #[error("{0}")]
    Conflict(String),
}

impl IntoResponse for ChaptersError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        match self {
            ChaptersError::ChapterNotFound => {
                (StatusCode::NOT_FOUND, Json(self.to_string())).into_response()
            }
            ChaptersError::BadRequest => {
                (StatusCode::BAD_REQUEST, Json(self.to_string())).into_response()
            }
            ChaptersError::ImageTooLarge => {
                (StatusCode::BAD_REQUEST, Json(self.to_string())).into_response()
            }
            ChaptersError::Conflict(_) => {
                (StatusCode::CONFLICT, Json(self.to_string())).into_response()
            }
            ChaptersError::Diesel(diesel_error) => {
                if let DatabaseError(DatabaseErrorKind::UniqueViolation, message) = diesel_error {
                    let constraint_name = message.constraint_name();
                    match constraint_name {
                        Some("comic_chapters_comic_id_number_key") => {
                            return (
                                StatusCode::CONFLICT,
                                Json("chapter with same number already exists"),
                            )
                                .into_response();
                        }
                        Some("chapter_pages_chapter_id_number_key") => {
                            return (
                                StatusCode::CONFLICT,
                                Json("chapter page with same number already exists"),
                            )
                                .into_response();
                        }
                        _ => {}
                    }
                } else if let DatabaseError(DatabaseErrorKind::ForeignKeyViolation, message) =
                    diesel_error
                {
                    let constraint_name = message.constraint_name();
                    if let Some("comic_chapters_comic_id_fkey") = constraint_name {
                        return (StatusCode::NOT_FOUND, Json("comic not found")).into_response();
                    }
                }
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            ChaptersError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            ChaptersError::PoolError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            ChaptersError::Validator(errors) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: String::from("invalid input"),
                    details: Some(
                        errors
                            .flatten()
                            .iter()
                            .map(|(path, error)| format!("{path}: {error}"))
                            .collect::<Vec<String>>(),
                    ),
                },
            )
                .into_response(),
        }
    }
}
