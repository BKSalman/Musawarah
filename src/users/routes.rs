use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use jwt_simple::prelude::*;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    posts::models::{ImageResponse, PostResponse},
    PaginationParams, JWT_KEY,
};

use super::{
    models::{CreateUser, UserClaims, UserLogin, UserResponse, UserToken},
    UsersError,
};

/// Create User
#[utoipa::path(
    post,
    path = "/api/users/",
    request_body(content = CreateUser, description = "Username, Email, and password", content_type = "application/json"),
    responses(
        (status = 200, description = "User successfully created", body = UserResponse),
        (status = StatusCode::BAD_REQUEST, description = "Fields validation error", body = ErrorHandlingResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Users API"
)]
pub async fn create_user(
    State(db): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<UserResponse>, UsersError> {
    payload.validate()?;

    if payload.username.is_empty() || payload.password.is_empty() || payload.email.is_empty() {
        return Err(UsersError::BadRequest);
    }

    // v7 uuid allows for easier sorting
    let uuid = Uuid::now_v7();

    let salt = SaltString::new("somethingsomething").map_err(|err| {
        tracing::debug!("salt string error: {:#?}", err);
        UsersError::InternalServerError
    })?;

    // argon2 is a good algorithm (not a security expert :))
    let argon2 = Argon2::default();

    let hashed_password = argon2
        .hash_password(payload.password.as_bytes(), &salt)?
        .to_string();

    let user = sqlx::query!(
        r#"
INSERT INTO users ( id, username, displayname, email, password )
VALUES ( $1, $2, $3, $4 , $5)

RETURNING *
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
            _ => {
                tracing::debug!("create_user db error: {:#?}", dbe);
                UsersError::InternalServerError
            }
        },
        _ => {
            // TODO: log this instead of printing
            tracing::debug!("{e:#?}");
            UsersError::InternalServerError
        }
    })?;

    let user = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
    };

    Ok(Json(user))
}

/// User login
#[utoipa::path(
    post,
    path = "/api/users/login",
    request_body(content = UserLogin, description = "Email and password", content_type = "application/json"),
    responses(
        (status = 200, description = "User authenticated", body = UserToken),
        (status = StatusCode::UNAUTHORIZED, description = "User unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Users API"
)]
pub async fn login(
    State(db): State<PgPool>,
    Json(payload): Json<UserLogin>,
) -> Result<Json<UserToken>, UsersError> {
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
        _ => {
            tracing::debug!("login db error: {:#?}", error);
            UsersError::InternalServerError
        }
    })?;

    let Some(user) = record else {
        return Err(UsersError::UserNotFound);
    };

    let parsed_password = PasswordHash::new(&user.password)?;

    if argon2
        .verify_password(payload.password.as_bytes(), &parsed_password)
        .is_err()
    {
        return Err(UsersError::InvalidCredentials);
    }

    let claims = Claims::with_custom_claims(
        UserClaims {
            user: UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
            },
        },
        Duration::from_mins(20),
    );

    let token = JWT_KEY.authenticate(claims)?;

    Ok(Json(UserToken {
        access_token: token,
        r#type: String::from("Bearer"),
    }))
}

/// Get user posts by username
#[utoipa::path(
    get,
    path = "/api/users/{username}",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "Caller authorized. returned requested user's posts", body = [PostResponse]),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users API"
)]
pub async fn get_user_posts(
    // prevent non logged users from
    // accessing a specific user's posts
    _: UserClaims,
    State(db): State<PgPool>,
    Path(username): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<PostResponse>>, UsersError> {
    // TODO: cursor shit
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

WHERE username = $1 AND posts.id > $2 AND posts.id < $3

LIMIT 10
        "#,
        username,
        pagination.min_id,
        pagination.max_id,
    )
    .fetch_all(&db)
    .await
    // TODO: better error handling
    .map_err(|_| UsersError::UserNotFound)?;

    let posts = records
        .into_iter()
        .map(|r| PostResponse {
            id: r.post_id,
            title: r.title,
            content: r.content,
            created_at: r.created_at.to_string(),
            user: UserResponse {
                id: r.user_id,
                username: r.username,
                email: r.email,
            },
            image: ImageResponse {
                content_type: r.content_type,
                path: r.path,
            },
        })
        .collect::<Vec<PostResponse>>();

    if posts.len() == 0 {
        return Err(UsersError::UserNotFound);
    }

    Ok(Json(posts))
}

/// Get user details by token
#[utoipa::path(
    get,
    path = "/api/users/",
    responses(
        (status = 200, description = "Caller authorized. returned current user info", body = UserClaims),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users API"
)]
pub async fn get_user(claims: UserClaims) -> Result<Json<UserClaims>, UsersError> {
    Ok(Json(claims))
}
