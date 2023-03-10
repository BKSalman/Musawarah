use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use axum::{
    extract::{Path, State},
    Json,
};
use jwt_simple::prelude::*;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    posts::models::{ImageResponse, PostResponse},
    JWT_KEY,
};

use super::{
    models::{CreateUser, UserClaims, UserLogin, UserReponse},
    UsersError,
};

pub async fn create_user(
    State(db): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<Value>, UsersError> {
    payload.validate()?;

    if payload.username.is_empty() || payload.password.is_empty() || payload.email.is_empty() {
        return Err(UsersError::BadRequest);
    }

    // v7 uuid allows for easier sorting
    let uuid = Uuid::now_v7();

    let salt = SaltString::new("somethingsomething").expect("salt");

    // argon2 is a good algorithm (not a security expert :))
    let argon2 = Argon2::default();

    let hashed_password = argon2
        .hash_password(payload.password.as_bytes(), &salt)?
        .to_string();

    let _id = sqlx::query!(
        r#"
INSERT INTO users ( id, username, displayname, email, password )
VALUES ( $1, $2, $3, $4 , $5)
RETURNING id
            "#,
        uuid,
        payload.username.to_lowercase(),
        payload.username,
        payload.email,
        hashed_password,
    )
    .fetch_one(&db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(dbe) => match dbe.constraint() {
            Some("users_username_key") => UsersError::Conflict("username taken".into()),
            Some("users_email_key") => UsersError::Conflict("email taken".into()),
            _ => UsersError::InternalServerError,
        },
        _ => {
            // TODO: log this instead of printing
            println!("{e:#?}");
            UsersError::InternalServerError
        }
    })?
    .id;

    // let user = UserReponse {
    //     id: id.to_string(),
    //     username: payload.username.to_lowercase(),
    // };

    Ok(Json(json!({"message": "registered successfully"})))
}

pub async fn login(
    State(db): State<PgPool>,
    Json(payload): Json<UserLogin>,
) -> Result<Json<Value>, UsersError> {
    // argon2 is a good algorithm (not a security expert :))
    let argon2 = Argon2::default();

    let record = sqlx::query!(
        r#"
SELECT users.*
FROM users
WHERE email = $1;
        "#,
        payload.email,
    )
    .fetch_optional(&db)
    .await
    // TODO: better error handling
    .map_err(|error| match error {
        _ => UsersError::InternalServerError,
    })?;

    let Some(user) = record else {
        return Err(UsersError::UserNotFound);
    };

    let parsed_password = PasswordHash::new(&user.password)?;

    if argon2
        .verify_password(payload.password.as_bytes(), &parsed_password)
        .is_err()
    {
        return Err(UsersError::WrongPassword);
    }

    let claims = Claims::with_custom_claims(
        UserClaims {
            user: UserReponse {
                id: user.id.to_string(),
                username: user.username,
                email: user.email,
            },
        },
        Duration::from_mins(20),
    );

    let token = JWT_KEY.authenticate(claims)?;

    Ok(Json(json!({
        "access_token": token,
        "type": "Bearer",
    })))
}

pub async fn get_user_posts(
    // prevent non logged users from
    // accessing a specific user's posts
    _: UserClaims,
    State(db): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<Vec<PostResponse>>, UsersError> {
    let records = sqlx::query!(
        r#"
SELECT users.id AS user_id, posts.id AS post_id,
users.displayname, users.username, users.email,
posts.content, posts.title, posts.created_at,
images.path, images.content_type, images.id AS image_id

FROM posts
INNER JOIN users
ON users.id = posts.author_id
INNER JOIN images
ON posts.id = images.post_id
WHERE username = $1;
        "#,
        username
    )
    .fetch_all(&db)
    .await
    // TODO: better error handling
    .map_err(|_| UsersError::UserNotFound)?;

    let posts = records
        .into_iter()
        .map(|r| PostResponse {
            id: r.post_id.to_string(),
            title: r.title,
            content: r.content,
            created_at: r.created_at.to_string(),
            user: UserReponse {
                id: r.user_id.to_string(),
                username: r.username,
                email: r.email,
            },
            image: ImageResponse {
                content_type: r.content_type,
                path: r.path,
            },
        })
        .collect::<Vec<PostResponse>>();

    Ok(Json(posts))
}

/// get user details for profile
pub async fn get_user(claims: UserClaims) -> Result<Json<UserClaims>, UsersError> {
    Ok(Json(claims))
}
