use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::ErrorResponse;

pub mod models;
pub mod routes;

#[derive(thiserror::Error, Debug)]
pub enum UsersError {
    #[error("internal server error")]
    InternalServerError,

    #[error("user not found")]
    UserNotFound,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("bad request")]
    BadRequest,

    #[error("user has no posts")]
    HasNoPosts,

    #[error("already logged in")]
    AlreadyLoggedIn,

    #[error("internal server error")]
    Diesel(#[from] diesel::result::Error),

    #[error("internal server error")]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error("internal server error")]
    Argon2(#[from] argon2::password_hash::Error),

    #[error("validation error: {0}")]
    Validator(#[from] garde::Errors),

    #[error("{0}")]
    Conflict(String),
}

impl IntoResponse for UsersError {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("{}", self.to_string());

        let (status, error_message) = match self {
            UsersError::UserNotFound => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::HasNoPosts => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    errors: vec![String::from("user has no posts")],
                },
            ),
            UsersError::BadRequest => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::Conflict(_) => (
                StatusCode::CONFLICT,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::Diesel(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::Argon2(_) => {
                // TODO: add logging for this
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        errors: vec![self.to_string()],
                    },
                )
            }
            UsersError::AlreadyLoggedIn => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::Validator(errors) => {
                let errors = errors
                    .flatten()
                    .iter()
                    .map(|(path, error)| format!("{path}: {error}"))
                    .collect::<Vec<String>>();

                (StatusCode::BAD_REQUEST, ErrorResponse { errors })
            }
            UsersError::PoolError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
        };

        let body = Json(error_message);

        (status, body).into_response()
    }
}
