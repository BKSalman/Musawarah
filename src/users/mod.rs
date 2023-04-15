use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::ErrorHandlingResponse;

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

    #[error("internal server error")]
    JWT(#[from] jwt_simple::Error),

    #[error("internal server error")]
    Sqlx(#[from] sqlx::Error),

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
                ErrorHandlingResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::HasNoPosts => (
                StatusCode::NOT_FOUND,
                ErrorHandlingResponse {
                    errors: vec![String::from("user has no posts")],
                },
            ),
            UsersError::BadRequest => (
                StatusCode::BAD_REQUEST,
                ErrorHandlingResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::Conflict(_) => (
                StatusCode::CONFLICT,
                ErrorHandlingResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                ErrorHandlingResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::Sqlx(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorHandlingResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorHandlingResponse {
                    errors: vec![self.to_string()],
                },
            ),
            UsersError::Argon2(_) => {
                // TODO: add logging for this
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorHandlingResponse {
                        errors: vec![self.to_string()],
                    },
                )
            }
            UsersError::JWT(_) => {
                // TODO: add logging for this
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorHandlingResponse {
                        errors: vec![self.to_string()],
                    },
                )
            }
            UsersError::Validator(errors) => {
                let errors = errors
                    .flatten()
                    .iter()
                    .map(|(path, error)| format!("{path}: {error}"))
                    .collect::<Vec<String>>();

                (StatusCode::BAD_REQUEST, ErrorHandlingResponse { errors })
            }
        };

        let body = Json(error_message);

        (status, body).into_response()
    }
}
