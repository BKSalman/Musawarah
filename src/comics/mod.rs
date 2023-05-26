use axum::{http::StatusCode, response::IntoResponse, Json};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use diesel_async::pooled_connection::deadpool::PoolError;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

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

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    PoolError(#[from] PoolError),

    // #[error("validation error: {0}")]
    // Validator(#[from] validator::ValidationErrors),
    #[error("{0}")]
    ComicGenresErrors(#[from] crate::comic_genres::ComicGenresError),
}

impl IntoResponse for ComicsError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        match self {
            ComicsError::ComicNotFound => StatusCode::NOT_FOUND.into_response(),
            ComicsError::BadRequest => StatusCode::BAD_REQUEST.into_response(),
            ComicsError::ImageTooLarge => StatusCode::BAD_REQUEST.into_response(),
            ComicsError::Diesel(diesel_error) => {
                if let DatabaseError(DatabaseErrorKind::UniqueViolation, message) = diesel_error {
                    let constraint_name = message.constraint_name().unwrap();
                    if constraint_name == "comics_title_key" {
                        return (
                            StatusCode::CONFLICT,
                            Json("comic with same title already exists"),
                        )
                            .into_response();
                    } else if constraint_name == "comic_genres_mapping_pkey" {
                        return (StatusCode::CONFLICT, Json("duplicate comic genre"))
                            .into_response();
                    }
                }
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            ComicsError::PoolError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            ComicsError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            ComicsError::ComicGenresErrors(_) => StatusCode::BAD_REQUEST.into_response(),
        }
    }
}
