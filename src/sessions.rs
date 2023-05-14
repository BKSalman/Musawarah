use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{entity, AppState, ErrorHandlingResponse, COOKIES_SECRET};

pub const SESSION_COOKIE_NAME: &str = "session_id";

pub struct UserSession {
    session_id: Option<Uuid>,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum SessionError {
    #[error("something went wrong")]
    SomethingWentWrong,

    #[error("invalid session")]
    InvalidSession,
}

impl IntoResponse for SessionError {
    fn into_response(self) -> axum::response::Response {
        let (error_status, error_message) = match self {
            SessionError::SomethingWentWrong => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorHandlingResponse {
                    errors: vec![self.to_string()],
                },
            ),
            SessionError::InvalidSession => (
                StatusCode::UNAUTHORIZED,
                ErrorHandlingResponse {
                    errors: vec![self.to_string()],
                },
            ),
        };

        let body = Json(error_message);

        (error_status, body).into_response()
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
    State(db): State<DatabaseConnection>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, SessionError> {
    tracing::info!("running refresh_session middleware");
    if let Some(session_id) = session.session_id {
        let session = entity::sessions::ActiveModel {
            id: ActiveValue::Set(session_id),
            // refresh session
            expires_at: ActiveValue::Set((Utc::now() + Duration::days(2)).naive_utc()),
            ..Default::default()
        };

        session.update(&db).await.map_err(|e| {
            tracing::error!("session-extractor: could not update session: {e}");
            SessionError::SomethingWentWrong
        })?;
    }

    Ok(next.run(request).await)
}
