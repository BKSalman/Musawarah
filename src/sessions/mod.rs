pub mod models;

use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    RequestPartsExt,
};
use chrono::{Duration, Utc};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection, RunQueryDsl};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{schema::sessions, sessions::models::Session, AppState, ErrorResponse, COOKIES_SECRET};

pub const SESSION_COOKIE_NAME: &str = "session_id";

pub struct UserSession {
    session_id: Option<Uuid>,
}

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("something went wrong")]
    SomethingWentWrong,

    #[error("something went wrong")]
    Diesel(#[from] diesel::result::Error),

    #[error("something went wrong")]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error("invalid session")]
    InvalidSession,
}

impl IntoResponse for SessionError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        match self {
            SessionError::SomethingWentWrong => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            SessionError::InvalidSession => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            SessionError::Diesel(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            SessionError::PoolError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for UserSession {
    type Rejection = SessionError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies =
            parts
                .extract::<Cookies>()
                .await
                .map_err(|(_error_status, error_message)| {
                    tracing::error!(
                        "session-extractor: failed to get private cookie jar: {error_message}"
                    );
                    SessionError::InvalidSession
                })?;

        let key = COOKIES_SECRET.get().expect("cookies secret key");

        if let Some(session_id) = cookies.private(key).get(SESSION_COOKIE_NAME) {
            Ok(Self {
                session_id: Some(Uuid::parse_str(session_id.value()).map_err(|e| {
                    tracing::error!("session-extractor: invalid session_id: {e}");
                    SessionError::InvalidSession
                })?),
            })
        } else {
            Ok(Self { session_id: None })
        }
    }
}

pub async fn refresh_session<B>(
    session: UserSession,
    State(pool): State<Pool<AsyncPgConnection>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, SessionError> {
    tracing::info!("running refresh_session middleware");

    let mut db = pool.get().await?;

    if let Some(session_id) = session.session_id {
        diesel::update(sessions::table.find(session_id))
            .set(sessions::expires_at.eq(Utc::now() + Duration::days(2)))
            .execute(&mut db)
            .await?;
    }

    Ok(next.run(request).await)
}
