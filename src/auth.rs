use async_trait::async_trait;
use axum::{
    extract::FromRequestParts, http::StatusCode, response::IntoResponse, Json, RequestPartsExt,
};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::{
    schema::{sessions, users},
    sessions::{models::Session, SESSION_COOKIE_NAME},
    users::models::{User, UserResponseBrief},
    AppState, ErrorResponse, COOKIES_SECRET,
};

pub struct AuthExtractor {
    pub current_user: UserResponseBrief,
    pub session_id: Uuid,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("something went wrong")]
    SomethinWentWrong,

    #[error("something went wrong")]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error("something went wrong")]
    Diesel(#[from] diesel::result::Error),

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
            AuthError::Diesel(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    errors: vec![self.to_string()],
                },
            ),
            AuthError::PoolError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
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
        let mut db = state.pool.get().await?;

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
        let mut remove_cookie_on_fail = Cookie::build(SESSION_COOKIE_NAME, "")
            .path("/")
            .http_only(true);

        #[cfg(not(debug_assertions))]
        {
            remove_cookie_on_fail = cookie_to_be_removed
                // TODO: use the actual musawarah domain
                .domain("salmanforgot.com")
                .secure(true);
        }

        #[cfg(debug_assertions)]
        {
            remove_cookie_on_fail = remove_cookie_on_fail.domain("localhost");
        }

        let session_id = cookies
            .private(key)
            .get(SESSION_COOKIE_NAME)
            .ok_or_else(|| {
                tracing::error!("auth-extractor: failed to get session_id cookie");
                cookies.remove(remove_cookie_on_fail.clone().finish());
                AuthError::InvalidSession
            })?;

        let session_id = Uuid::parse_str(session_id.value()).map_err(|e| {
            tracing::error!("auth-extractor: invalid session_id: {e}");
            // FIXME: why do I need to clone
            cookies.remove(remove_cookie_on_fail.clone().finish());
            AuthError::InvalidSession
        })?;

        let Ok((user, session)) = sessions::table
            .inner_join(users::table)
            .filter(sessions::id.eq(session_id))
            .filter(sessions::expires_at.gt(Utc::now()))
            .select((User::as_select(), Session::as_select()))
            .get_result::<(User, Session)>(&mut db)
            .await else {
            cookies.remove(remove_cookie_on_fail.clone().finish());
            return Err(AuthError::InvalidSession);
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
