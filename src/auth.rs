use async_trait::async_trait;
use axum::{
    extract::FromRequestParts, http::StatusCode, response::IntoResponse, Json, RequestPartsExt,
};
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::{
    entity, sessions::SESSION_COOKIE_NAME, users::models::UserResponseBrief, AppState,
    ErrorResponse, COOKIES_SECRET,
};

pub struct AuthExtractor {
    pub current_user: UserResponseBrief,
    pub session_id: Uuid,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum AuthError {
    #[error("something went wrong")]
    SomethinWentWrong,

    #[error("invalid session")]
    InvalidSession,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (error_status, error_message) = match self {
            AuthError::SomethinWentWrong => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            AuthError::InvalidSession => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
        };

        let body = Json(error_message);

        (error_status, body).into_response()
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthExtractor {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        let cookies =
            parts
                .extract::<Cookies>()
                .await
                .map_err(|(_error_status, error_message)| {
                    tracing::error!("auth-extractor: failed to get cookies: {error_message}");
                    AuthError::InvalidSession
                })?;

        let key = COOKIES_SECRET.get().expect("cookies secret key");

        #[allow(unused_mut)]
        let mut cookie_to_be_removed = Cookie::build(SESSION_COOKIE_NAME, "")
            .path("/")
            .http_only(true);

        #[cfg(not(debug_assertions))]
        {
            cookie_to_be_removed = cookie_to_be_removed
                // TODO: use the actual musawarah domain
                .domain("salmanforgot.com")
                .secure(true);
        }

        #[cfg(debug_assertions)]
        {
            cookie_to_be_removed = cookie_to_be_removed.domain("localhost");
        }

        let session_id = cookies
            .private(key)
            .get(SESSION_COOKIE_NAME)
            .ok_or_else(|| {
                tracing::error!("auth-extractor: failed to get session_id cookie");
                cookies.remove(cookie_to_be_removed.clone().finish());
                AuthError::InvalidSession
            })?;

        let session_id = Uuid::parse_str(session_id.value()).map_err(|e| {
            tracing::error!("auth-extractor: invalid session_id: {e}");
            // FIXME: this is weird
            cookies.remove(cookie_to_be_removed.clone().finish());
            AuthError::InvalidSession
        })?;

        let (session, Some(user)) = entity::sessions::Entity::find_by_id(session_id)
            .filter(entity::sessions::Column::ExpiresAt.gt(Utc::now().naive_utc()))
            .find_also_related(entity::users::Entity)
            .one(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("auth-extractor: failed to fetch session: {e}");
                cookies.remove(cookie_to_be_removed.clone().finish());
                AuthError::InvalidSession
            })?
            .ok_or_else(|| {
                tracing::error!("auth-extractor: failed to fetch active (not expired) session");
                cookies.remove(cookie_to_be_removed.finish());
                AuthError::InvalidSession
            })? else {
                tracing::error!("auth-extractor: failed to fetch user");
                return Err(AuthError::SomethinWentWrong);
            };

        Ok(AuthExtractor {
            current_user: UserResponseBrief {
                id: user.id,
                displayname: user.displayname,
                username: user.username,
                email: user.email,
            },
            session_id: session.id,
        })
    }
}
