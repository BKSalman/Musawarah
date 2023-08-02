use axum::{http::StatusCode, response::IntoResponse};
use diesel::result::DatabaseErrorKind;

use crate::ErrorResponse;

pub mod models;
pub mod routes;

#[derive(Debug, thiserror::Error)]
pub enum ChapterCommentsError {
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
}

impl IntoResponse for ChapterCommentsError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        match self {
            ChapterCommentsError::Diesel(diesel_err) => {
                if let diesel::result::Error::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    message,
                ) = diesel_err
                {
                    let Some(constraint) = message.constraint_name() else {
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    };

                    if constraint == "chapter_comments_chapter_id_fkey" {
                        return (
                            StatusCode::BAD_REQUEST,
                            ErrorResponse {
                                error: String::from("chapter not found"),
                                ..Default::default()
                            },
                        )
                            .into_response();
                    }
                } else if diesel_err == diesel::result::Error::NotFound {
                    return (
                        StatusCode::NOT_FOUND,
                        ErrorResponse {
                            error: String::from("comment id not found"),
                            ..Default::default()
                        },
                    )
                        .into_response();
                }
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
