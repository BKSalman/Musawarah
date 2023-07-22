use std::sync::Arc;

use axum::extract::Path;
use axum::routing::post;
use axum::{extract::State, Router};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::users;
use crate::InnerAppState;
use crate::{auth::AuthExtractor, schema::email_verifications, users::models::UserRole, AppState};

use super::{models::EmailVerification, EmailVerificationError};

pub fn email_verification_router() -> Router<AppState> {
    Router::new()
        .route("/email-verification", post(create_email_verification))
        .route("/confirm-email/:verification_id", post(confirm_email))
}

/// Send email
#[utoipa::path(
    post,
    path = "/api/v1/users/email_verification",
    responses(
        (status = 200, description = "Verification email sent"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
        (status = StatusCode::BAD_REQUEST, description = "User Already verified", body = ErrorResponse),
    ),
    tag = "Email Verification API"
)]
pub async fn create_email_verification(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
) -> Result<(), EmailVerificationError> {
    if auth.current_user.role != UserRole::User {
        return Err(EmailVerificationError::AlreadyVerified);
    }
    let mut db = state.pool.get().await?;
    let email_verification = EmailVerification {
        id: Uuid::now_v7(),
        email: auth.current_user.email,
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(1),
        user_id: auth.current_user.id,
    };
    diesel::insert_into(email_verifications::table)
        .values(&email_verification)
        .execute(&mut db)
        .await?;
    email_verification
        .send_email(auth.current_user.username, state)
        .await?;
    Ok(())
}

// Verify Email
#[utoipa::path(
    post,
    path = "/api/v1/users/confirm_email/:verification_id",
    responses(
        (status = 200, description = "User's email has been verified"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
        (status = StatusCode::NOT_FOUND, description = "No email with this id has been found", body = ErrorResponse),
        (status = StatusCode::GONE, description = "Email has expired", body = ErrorResponse),
        (status = StatusCode::BAD_REQUEST, description = "User Already verified", body = ErrorResponse),
    ),
    tag = "Email Verification API"
)]
pub async fn confirm_email(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(verification_id): Path<Uuid>,
) -> Result<(), EmailVerificationError> {
    if auth.current_user.role != UserRole::User {
        return Err(EmailVerificationError::AlreadyVerified);
    }
    let mut db = state.pool.get().await?;
    let email_verification: EmailVerification = diesel::delete(
        email_verifications::table.filter(email_verifications::id.eq(&verification_id)),
    )
    .returning(EmailVerification::as_returning())
    .get_result(&mut db)
    .await?;

    if email_verification.expires_at < Utc::now() {
        return Err(EmailVerificationError::ExpiredEmail);
    }

    diesel::update(users::table.find(email_verification.user_id))
        .set(users::role.eq(UserRole::VerifiedUser))
        .execute(&mut db)
        .await?;
    Ok(())
}
