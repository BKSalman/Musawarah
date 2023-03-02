use std::collections::HashMap;

use axum::{extract::Path, Extension, Json};
use sqlx::PgPool;

use super::{
    models::{CreateUser, UserReponse},
    UsersError,
};

pub async fn create_user(
    db: Extension<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<UserReponse>, UsersError> {
    if payload.username.is_empty() || payload.password.is_empty() || payload.email.is_empty() {
        return Err(UsersError::Bad);
    };

    let id = sqlx::query!(
        r#"
    INSERT INTO users ( username, displayname, email, password )
    VALUES ( $1, $2, $3, $4 )
    RETURNING id
            "#,
        payload.username.to_lowercase(),
        payload.username,
        payload.email,
        payload.password,
    )
    .fetch_one(&*db)
    .await
    // TODO: better error handling
    .map_err(|_| UsersError::FailedInsert)?
    .id;

    let user = UserReponse {
        id: id.to_string(),
        username: payload.username.to_lowercase(),
    };

    Ok(Json(user))
}

pub async fn get_user(
    Path(params): Path<HashMap<String, String>>,
    db: Extension<PgPool>,
) -> Result<Json<UserReponse>, UsersError> {
    let Some(username) = params.get("username") else {
        return Err(UsersError::MissingUsernameParam)
    };

    let user = sqlx::query!(
        r#"
SELECT * FROM users WHERE username = $1
        "#,
        username
    )
    .fetch_one(&*db)
    .await
    // TODO: better error handling
    .map_err(|_| UsersError::UserNotFound)?;

    let user = UserReponse {
        id: user.id.to_string(),
        username: user.username,
    };

    Ok(Json(user))
}
