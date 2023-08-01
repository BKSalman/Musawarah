use async_trait::async_trait;
use axum::{extract::FromRequestParts, http::StatusCode, response::IntoResponse, RequestPartsExt};
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    schema::{sessions, users},
    sessions::{models::Session, UserSession},
    users::models::{User, UserResponseBrief, UserRole},
    AppState, ErrorResponse,
};

// TODO: add generic for UserRole
// this will allow for role checking
pub struct AuthExtractor<const USER_ROLE: u32> {
    pub current_user: UserResponseBrief,
    pub session_id: Uuid,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("something went wrong")]
    SomethinWentWrong,

    #[error(transparent)]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error("invalid session")]
    InvalidSession,

    #[error("invalid session")]
    SessionError(#[from] crate::sessions::SessionError),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        match self {
            AuthError::SomethinWentWrong => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            AuthError::InvalidSession => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            AuthError::Diesel(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            AuthError::PoolError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            AuthError::SessionError(e) => e.into_response(),
        }
    }
}

#[async_trait]
impl<const USER_ROLE: u32> FromRequestParts<AppState> for AuthExtractor<USER_ROLE> {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        let mut db = state.inner.pool.get().await?;

        let session_id = parts
            .extract_with_state::<UserSession, _>(state)
            .await?
            .session_id
            .ok_or_else(|| {
                tracing::error!("auth-extractor: missing session_id");
                AuthError::InvalidSession
            })?;

        let mut query = sessions::table
            .inner_join(users::table)
            .filter(sessions::id.eq(session_id))
            .filter(sessions::expires_at.gt(Utc::now()))
            .into_boxed();

        // Safety: USER_ROLE is only provided by casting UserRole variants
        let role: UserRole = unsafe { std::mem::transmute(USER_ROLE) };

        match role {
            UserRole::Admin => query = query.filter(users::role.eq(role)),
            UserRole::Staff => {
                query = query.filter(users::role.eq(role).or(users::role.eq(UserRole::Admin)))
            }
            UserRole::VerifiedUser => {
                query = query.filter(
                    users::role.eq(role).or(users::role
                        .eq(UserRole::Admin)
                        .or(users::role.eq(UserRole::Staff))),
                )
            }
            UserRole::User => {}
        };

        let Ok((user, session)) = query
            .select((User::as_select(), Session::as_select()))
            .get_result::<(User, Session)>(&mut db)
            .await else {
            diesel::delete(sessions::table.filter(sessions::id.eq(session_id))).execute(&mut db).await?;
            return Err(AuthError::InvalidSession);
        };

        Ok(AuthExtractor {
            current_user: UserResponseBrief {
                id: user.id,
                displayname: user.displayname,
                username: user.username,
                email: user.email,
                role: user.role,
            },
            session_id: session.id,
        })
    }
}
