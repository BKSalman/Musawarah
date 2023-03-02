use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub mod models;
pub mod routes;

pub enum UsersError {
    Bad,
    UserNotFound,
    MissingUsernameParam,
    FailedInsert,
}

impl IntoResponse for UsersError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            UsersError::Bad => (StatusCode::BAD_REQUEST, "LMAO"),
            UsersError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            UsersError::MissingUsernameParam => {
                (StatusCode::BAD_REQUEST, "missing username parameter")
            }
            UsersError::FailedInsert => (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong"),
        };

        let body = Json(json!({
            "message": error_message,
        }));

        (status, body).into_response()
    }
}
