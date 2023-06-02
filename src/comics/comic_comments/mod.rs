use axum::{http::StatusCode, response::IntoResponse};

pub mod models;
pub mod routes;

#[derive(thiserror::Error, Debug)]
pub enum ComicCommentsError {
    #[error("something")]
    PlaceHolder,

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
}

impl IntoResponse for ComicCommentsError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        match self {
            ComicCommentsError::PlaceHolder => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            ComicCommentsError::Diesel(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
