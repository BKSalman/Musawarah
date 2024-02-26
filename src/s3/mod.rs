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

    #[error(transparent)]
    AWSGetError(#[from] aws_smithy_http::result::SdkError<aws_sdk_s3::error::GetObjectError>),

    #[error(transparent)]
    AWSPutError(#[from] aws_smithy_http::result::SdkError<aws_sdk_s3::error::PutObjectError>),

    #[error(transparent)]
    AWSStreamError(#[from] aws_smithy_http::byte_stream::error::Error),
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

            ImagesError::AWSGetError(e) => match e {
                aws_sdk_s3::types::SdkError::ServiceError(service_error) => {
                    match service_error.err().kind {
                        aws_sdk_s3::error::GetObjectErrorKind::NoSuchKey(_) => {
                            (StatusCode::BAD_REQUEST).into_response()
                        }
                        _ => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
                    }
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            },
            ImagesError::AWSPutError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            ImagesError::AWSStreamError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}
