use axum::extract::Path;
use axum::routing::post;
use axum::Json;
use axum::{extract::State, Router};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::schema::users;
use crate::{auth::AuthExtractor, schema::email_verifications, users::models::UserRole, AppState};

use super::{models::EmailVerification, EmailVerificationError};

pub fn email_verification_router() -> Router<AppState> {
    Router::new()
        .route("/email_verification", post(create_email_verification))
        .route("/confirm_email/:verification_id", post(confirm_email))
}

pub async fn create_email_verification(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(pool): State<Pool<AsyncPgConnection>>,
) -> Result<Json<Uuid>, EmailVerificationError> {
    let mut db = pool.get().await?;
    let verification_id = Uuid::now_v7();
    let email_verification = EmailVerification {
        id: verification_id,
        email: auth.current_user.email,
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(1),
        user_id: auth.current_user.id,
    };
    diesel::insert_into(email_verifications::table)
        .values(&email_verification)
        .returning(EmailVerification::as_returning())
        .execute(&mut db)
        .await?;
    // TODO: send email
    Ok(Json(verification_id))
}

pub async fn confirm_email(
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(verification_id): Path<Uuid>,
) -> Result<(), EmailVerificationError> {
    let mut db = pool.get().await?;
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
