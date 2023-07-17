use axum::{http::StatusCode, response::IntoResponse};

use crate::ErrorResponse;

pub mod models;
pub mod routes;

#[derive(thiserror::Error, Debug)]
pub enum EmailVerificationError {
    #[error(transparent)]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error("something went wrong")]
    Diesel(#[from] diesel::result::Error),

    #[error("Email has expired")]
    ExpiredEmail,
}

impl IntoResponse for EmailVerificationError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        match self {
            Self::Diesel(diesel_error) => {
                if let diesel::result::Error::NotFound = diesel_error {
                    return (
                        StatusCode::NOT_FOUND,
                        ErrorResponse {
                            error: String::from("verification id not found"),
                            ..Default::default()
                        },
                    )
                        .into_response();
                }
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            Self::PoolError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            Self::ExpiredEmail => (
                StatusCode::GONE,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
        }
    }
}
