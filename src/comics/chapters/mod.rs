pub mod chapter_comments;
pub mod models;
pub mod routes;
mod utils;

use axum::{http::StatusCode, response::IntoResponse};
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
            ChaptersError::ChapterNotFound => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            ChaptersError::BadRequest => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            ChaptersError::ImageTooLarge => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            ChaptersError::Conflict(_) => (
                StatusCode::CONFLICT,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            ChaptersError::Diesel(diesel_error) => match diesel_error {
                diesel::result::Error::NotFound => StatusCode::NOT_FOUND.into_response(),
                DatabaseError(DatabaseErrorKind::UniqueViolation, message) => {
                    let constraint_name = message.constraint_name();
                    match constraint_name {
                        Some("comic_chapters_comic_id_number_key") => (
                            StatusCode::CONFLICT,
                            ErrorResponse {
                                error: String::from("chapter with same number already exists"),
                                ..Default::default()
                            },
                        )
                            .into_response(),
                        Some("chapter_pages_chapter_id_number_key") => (
                            StatusCode::CONFLICT,
                            ErrorResponse {
                                error: String::from("chapter page with same number already exists"),
                                ..Default::default()
                            },
                        )
                            .into_response(),
                        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                    }
                }
                DatabaseError(DatabaseErrorKind::ForeignKeyViolation, message) => {
                    let constraint_name = message.constraint_name();
                    if let Some("comic_chapters_comic_id_fkey") = constraint_name {
                        return (
                            StatusCode::NOT_FOUND,
                            ErrorResponse {
                                error: String::from("comic not found"),
                                ..Default::default()
                            },
                        )
                            .into_response();
                    }
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                diesel::result::Error::QueryBuilderError(message) => {
                    let message = message.to_string();

                    if message.contains("no changes")
                    // "There are no changes to save. This query cannot be built"
                    {
                        return (
                            StatusCode::BAD_REQUEST,
                            ErrorResponse {
                                error: String::from("no changes"),
                                ..Default::default()
                            },
                        )
                            .into_response();
                    }
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            },
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
