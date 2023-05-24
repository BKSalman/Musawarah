pub mod models;
pub mod routes;
mod utils;

use axum::{http::StatusCode, response::IntoResponse, Json};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use diesel_async::pooled_connection::deadpool::PoolError;
// use tracing::debug;

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

    #[error("internal server error")]
    Diesel(#[from] diesel::result::Error),

    #[error("internal server error")]
    PoolError(#[from] PoolError),

    // #[error("validation error: {0}")]
    // Validator(#[from] validator::ValidationErrors),
    #[error("{0}")]
    Conflict(String),
}

impl IntoResponse for ChaptersError {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("{:#?}", self);

        match self {
            ChaptersError::ChapterNotFound => {
                (StatusCode::NOT_FOUND, Json(vec![self.to_string()])).into_response()
            }
            ChaptersError::BadRequest => {
                (StatusCode::BAD_REQUEST, Json(vec![self.to_string()])).into_response()
            }
            ChaptersError::ImageTooLarge => {
                (StatusCode::BAD_REQUEST, Json(vec![self.to_string()])).into_response()
            }
            ChaptersError::Conflict(_) => {
                (StatusCode::CONFLICT, Json(vec![self.to_string()])).into_response()
            }
            ChaptersError::Diesel(diesel_error) => {
                if let DatabaseError(DatabaseErrorKind::UniqueViolation, message) = diesel_error {
                    let constraint_name = message.constraint_name();
                    if let Some("comic_chapters_comic_id_number_key") = constraint_name {
                        return (
                            StatusCode::CONFLICT,
                            Json("chapter with same number already exists"),
                        )
                            .into_response();
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
            ChaptersError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(vec![self.to_string()]),
            )
                .into_response(),
            ChaptersError::PoolError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(vec![self.to_string()]),
            )
                .into_response(),
        }
    }
}
