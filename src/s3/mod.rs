use axum::{http::StatusCode, response::IntoResponse, BoxError};
use derive_builder::Builder;
use serde::Deserialize;

use crate::ErrorResponse;

pub mod helpers;
pub mod interface;
pub mod models;
pub mod routes;

pub type Result<T, E = BoxError> = std::result::Result<T, E>;

#[derive(Builder, Deserialize)]
#[builder(pattern = "owned")]
pub struct Upload<S> {
    pub path: String,
    pub content_type: String,
    pub stream: S,
}

impl<S> Upload<S> {
    #[must_use]
    pub fn builder() -> UploadBuilder<S> {
        UploadBuilder::default()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ImagesError {
    #[error("internal server error")]
    InternalServerError,

    #[error("bad request")]
    BadRequest,
}

impl IntoResponse for ImagesError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        match self {
            ImagesError::BadRequest => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            ImagesError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}
