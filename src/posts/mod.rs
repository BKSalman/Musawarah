use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub mod models;
pub mod routes;
mod utils;

#[derive(thiserror::Error, Debug)]
pub enum PostsError {
    #[error("internal server error")]
    InternalServerError,

    #[error("post not found")]
    PostNotFound,

    #[error("bad request")]
    BadRequest,

    #[error("image size too large, maximum image size is 5MB")]
    ImageTooLarge,

    #[error("jwt internal server error")]
    JWT(#[from] jwt_simple::Error),

    #[error("sqlx internal server error")]
    Sqlx(#[from] sqlx::Error),

    #[error("validation error: {0}")]
    Validator(#[from] validator::ValidationErrors),

    #[error("{0}")]
    Conflict(String),
}

impl IntoResponse for PostsError {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("{}", self.to_string());

        let (status, error_message) = match self {
            PostsError::PostNotFound => {
                (StatusCode::NOT_FOUND, json!({"errors": [self.to_string()]}))
            }
            PostsError::BadRequest => (
                StatusCode::BAD_REQUEST,
                json!({"errors": [self.to_string()]}),
            ),
            PostsError::ImageTooLarge => (
                StatusCode::BAD_REQUEST,
                json!({"errors": [self.to_string()]}),
            ),
            PostsError::Conflict(_) => {
                (StatusCode::CONFLICT, json!({"errors": [self.to_string()]}))
            }
            PostsError::Sqlx(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"errors": [self.to_string()]}),
            ),
            PostsError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"errors": [self.to_string()]}),
            ),
            PostsError::JWT(_) => {
                // TODO: add logging for this
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "errors": [self.to_string()] }),
                )
            }
            // TODO: edit this to posts
            PostsError::Validator(errors) => {
                let errors = errors
                    .errors()
                    .iter()
                    .map(|(field_name, field_error)| match field_error {
                        validator::ValidationErrorsKind::Field(errors) => errors
                            .iter()
                            .map(|error| match error.code.as_ref() {
                                "length" => format!(
                                    "{field_name} length: minimum = {}, maximum = {}",
                                    error
                                        .params
                                        .get("min")
                                        .expect("min username limit")
                                        .as_i64()
                                        .expect("min number"),
                                    error
                                        .params
                                        .get("max")
                                        .unwrap_or(&serde_json::Value::Number(i32::MAX.into()))
                                        .as_i64()
                                        .expect("max number"),
                                ),
                                "email" => String::from("email not valid"),
                                _ => todo!(),
                            })
                            .collect(),
                        _ => todo!(),
                    })
                    .collect::<Vec<String>>();

                (StatusCode::BAD_REQUEST, json!({ "errors": errors }))
            }
        };

        let body = Json(error_message);

        (status, body).into_response()
    }
}
