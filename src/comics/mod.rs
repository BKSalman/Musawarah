use axum::{http::StatusCode, response::IntoResponse};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use diesel_async::pooled_connection::deadpool::PoolError;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::{ErrorResponse, SortingOrder};

use self::chapters::routes::FILE_SIZE_LIMIT_MB;

pub mod chapters;
pub mod comic_comments;
pub mod comic_genres;
pub mod models;
pub mod routes;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ComicsParams {
    #[serde(default = "Uuid::nil")]
    pub min_id: Uuid,
    #[serde(default = "Uuid::max")]
    pub max_id: Uuid,
    #[serde(default)]
    pub genre: Option<i32>,
    #[serde(default)]
    pub sorting: Option<SortingOrder>,
}

#[derive(thiserror::Error, Debug)]
pub enum ComicsError {
    #[error("internal server error")]
    InternalServerError,

    #[error("comic not found")]
    ComicNotFound,

    #[error("bad request")]
    BadRequest,

    #[error("image size too large, maximum image size is {}MB", FILE_SIZE_LIMIT_MB)]
    ImageTooLarge,

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    PoolError(#[from] PoolError),

    #[error(transparent)]
    Validator(#[from] garde::Errors),

    #[error(transparent)]
    ComicGenresErrors(#[from] crate::comics::comic_genres::ComicGenresError),
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
                    return match constraint_name {
                        "comics_title_key" => (
                            StatusCode::CONFLICT,
                            ErrorResponse {
                                error: String::from("comic with same title already exists"),
                                ..Default::default()
                            },
                        )
                            .into_response(),
                        "comic_genres_mapping_pkey" => (
                            StatusCode::CONFLICT,
                            ErrorResponse {
                                error: String::from("duplicate comic genre"),
                                ..Default::default()
                            },
                        )
                            .into_response(),
                        "comic_ratings_comic_id_fkey" => (
                            StatusCode::NOT_FOUND,
                            ErrorResponse {
                                error: String::from("comic not found"),
                                ..Default::default()
                            },
                        )
                            .into_response(),
                        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                    };
                } else if let DatabaseError(DatabaseErrorKind::ForeignKeyViolation, message) =
                    diesel_error
                {
                    let constraint_name = message.constraint_name().unwrap();
                    if constraint_name == "comic_ratings_comic_id_fkey" {
                        return (
                            StatusCode::NOT_FOUND,
                            ErrorResponse {
                                error: String::from("comic not found"),
                                ..Default::default()
                            },
                        )
                            .into_response();
                    };
                }
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            ComicsError::PoolError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            ComicsError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            ComicsError::ComicGenresErrors(_) => StatusCode::BAD_REQUEST.into_response(),
            ComicsError::Validator(errors) => (
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
