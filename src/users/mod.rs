use axum::{http::StatusCode, response::IntoResponse, Json};

use serde_json::json;

pub mod models;
pub mod routes;

#[derive(thiserror::Error, Debug)]
pub enum UsersError {
    #[error("internal server error")]
    InternalServerError,

    #[error("user not found")]
    UserNotFound,

    #[error("wrong password")]
    WrongPassword,

    #[error("bad request")]
    BadRequest,

    #[error("internal server error")]
    JWT(#[from] jwt_simple::Error),

    #[error("internal server error")]
    Sqlx(#[from] sqlx::Error),

    #[error("internal server error")]
    Argon2(#[from] argon2::password_hash::Error),

    #[error("validation error: {0}")]
    Validator(#[from] validator::ValidationErrors),

    #[error("{0}")]
    Conflict(String),
}

impl IntoResponse for UsersError {
    fn into_response(self) -> axum::response::Response {
        println!("{}", self.to_string());

        let (status, error_message) = match self {
            UsersError::UserNotFound => {
                (StatusCode::NOT_FOUND, json!({"errors": [self.to_string()]}))
            }
            UsersError::BadRequest => (
                StatusCode::BAD_REQUEST,
                json!({"errors": [self.to_string()]}),
            ),
            UsersError::Conflict(_) => {
                (StatusCode::CONFLICT, json!({"errors": [self.to_string()]}))
            }
            UsersError::WrongPassword => (
                StatusCode::UNAUTHORIZED,
                json!({"errors": [self.to_string()]}),
            ),
            UsersError::Sqlx(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"errors": [self.to_string()]}),
            ),
            UsersError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"errors": [self.to_string()]}),
            ),
            UsersError::Argon2(_) => {
                // TODO: add logging for this
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "errors": [self.to_string()] }),
                )
            }
            UsersError::JWT(_) => {
                // TODO: add logging for this
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "errors": [self.to_string()] }),
                )
            }
            UsersError::Validator(errors) => {
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
