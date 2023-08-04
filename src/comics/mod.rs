use axum::{http::StatusCode, response::IntoResponse};
use chrono::{DateTime, Utc};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use diesel_async::pooled_connection::deadpool::PoolError;
use serde::Deserialize;
use serde::Serialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::{ErrorResponse, SortingOrder};

use self::chapters::routes::FILE_SIZE_LIMIT_MB;

pub mod chapters;
pub mod comic_comments;
pub mod comic_genres;
pub mod models;
pub mod routes;
mod utils;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ComicsParams {
    #[serde(default = "Uuid::max")]
    pub max_id: Uuid,
    #[serde(default)]
    pub genre: Option<i32>,
    #[serde(default)]
    pub sorting: Option<SortingOrder>,
    #[serde(default)]
    pub order: Order,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Latest(DateTime<chrono::Utc>),
    Best(f64),
}

impl Default for Order {
    fn default() -> Self {
        Self::Latest(Utc::now())
    }
}

impl Default for ComicsParams {
    fn default() -> Self {
        ComicsParams {
            max_id: Uuid::max(),
            genre: Option::default(),
            order: Order::default(),
            sorting: Option::default(),
        }
    }
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
                    let constraint_name = message
                        .constraint_name()
                        .expect("postgresql always provides the constraint name");
                    return match constraint_name {
                        "comics_user_id_slug_key" => (
                            StatusCode::CONFLICT,
                            ErrorResponse {
                                error: String::from("comic with the same title already exists"),
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
                    let constraint_name = message
                        .constraint_name()
                        .expect("postgresql always provides the constraint name");
                    return match constraint_name {
                        "comic_ratings_comic_id_fkey" => {
                            return (
                                StatusCode::NOT_FOUND,
                                ErrorResponse {
                                    error: String::from("comic not found"),
                                    ..Default::default()
                                },
                            )
                                .into_response();
                        }
                        "comic_genres_mapping_genre_id_fkey" => {
                            return (
                                StatusCode::BAD_REQUEST,
                                ErrorResponse {
                                    error: String::from(
                                        "no genre is associated with the provided id",
                                    ),
                                    ..Default::default()
                                },
                            )
                                .into_response();
                        }

                        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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
