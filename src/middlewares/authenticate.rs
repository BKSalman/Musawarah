use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json, RequestPartsExt, TypedHeader,
};
use jwt_simple::prelude::*;
use serde_json::json;

use crate::{users::models::UserClaims, JWT_KEY};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("invalid token")]
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                json!({
                    "errors": [self.to_string()]
                }),
            ),
        };

        let body = Json(error_message);

        (status, body).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserClaims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data = JWT_KEY
            .verify_token::<UserClaims>(bearer.token(), None)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.custom)
    }
}
