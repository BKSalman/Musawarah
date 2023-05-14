use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{Duration, Utc};
use garde::Validate;
use itertools::multizip;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    LoaderTrait, ModelTrait, QueryFilter, QuerySelect,
};
use time::OffsetDateTime;
use tower_cookies::{cookie::Cookie, Cookies};
use uuid::Uuid;

use crate::{
    auth::AuthExtractor,
    chapters::models::ChapterResponseBrief,
    comics::models::{ComicResponseBrief, ImageResponse},
    entity,
    sessions::SESSION_COOKIE_NAME,
    AppState, PaginationParams, COOKIES_SECRET,
};

use super::{
    models::{CreateUser, UserLogin, UserResponse, UserResponseBrief},
    UsersError,
};

/// Create User
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body(
        content = CreateUser,
        description = "Validation:\n- username: min = 5, max = 60\n- password: min = 8",
        content_type = "application/json"),
    responses(
        (status = 200, description = "User successfully created", body = UserReponse),
        (status = StatusCode::BAD_REQUEST, description = "Fields validation error", body = ErrorHandlingResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Users API"
)]
#[axum::debug_handler]
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<UserResponse>, UsersError> {
    payload.validate(&())?;
    if payload.username.is_empty() || payload.password.is_empty() || payload.email.is_empty() {
        return Err(UsersError::BadRequest);
    }

    let salt = SaltString::generate(rand::thread_rng());

    // argon2 is a good algorithm (not a security expert :))
    let argon2 = Argon2::default();

    let hashed_password = argon2
        .hash_password(payload.password.as_bytes(), &salt)?
        .to_string();

    let user = entity::users::Model {
        id: Uuid::now_v7(),
        username: payload.username.to_lowercase(),
        // TODO: why do I need to clone this?
        displayname: payload.username.clone(),
        email: payload.email.clone(),
        password: hashed_password,
        created_at: Utc::now().naive_utc(),
        last_login: None,
        // TODO: check this
        first_name: None,
        last_name: None,
        phone_number: None,
    }
    .into_active_model()
    .insert(&state.db)
    .await
    .map_err(|err| {
        // tracing::error!("error {:#?}", err);
        if let migration::DbErr::Query(sea_orm::RuntimeErr::SqlxError(sqlx::Error::Database(
            error,
        ))) = err
        {
            return match error.constraint() {
                Some("users_username_key") => {
                    tracing::error!("{}", error);
                    UsersError::Conflict(String::from("username already taken"))
                }
                Some("users_email_key") => {
                    tracing::error!("{}", error);
                    UsersError::Conflict(String::from("email already taken"))
                }
                _ => {
                    tracing::error!("sqlx error: {}", error);
                    UsersError::InternalServerError
                }
            };
        }
        tracing::error!("Db error: {}", err);
        UsersError::InternalServerError
    })?;

    let profile_image = entity::profile_images::Model {
        id: Uuid::now_v7(),
        user_id: user.id,
        path: String::from("ppL.webp"),
        content_type: String::from("image/webp"),
    }
    .into_active_model()
    .insert(&state.db)
    .await?;

    Ok(Json(UserResponse {
        id: user.id,
        displayname: user.displayname,
        username: user.username,
        email: user.email,
        profile_image: ImageResponse {
            path: profile_image.path,
            content_type: profile_image.content_type,
        },
    }))
}

/// User login
#[utoipa::path(
    post,
    path = "/api/v1/users/login",
    request_body(
        content = UserLogin,
        description = "Validation:- password: min = 8",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "User authenticated", body = UserToken),
        (status = StatusCode::UNAUTHORIZED, description = "User unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Users API"
)]
pub async fn login(
    State(db): State<DatabaseConnection>,
    cookies: Cookies,
    Json(payload): Json<UserLogin>,
) -> Result<(), UsersError> {
    let key = COOKIES_SECRET.get().expect("cookies secret key");

    if let Some(session_id) = cookies.private(key).get(SESSION_COOKIE_NAME) {
        tracing::error!("user already logged in with session id: {session_id}");
        return Err(UsersError::AlreadyLoggedIn);
    }

    payload.validate(&())?;

    // argon2 is a good algorithm (not a security expert :))
    let argon2 = Argon2::default();

    let Some(user) = entity::users::Entity::find()
        .filter(entity::users::Column::Email.eq(&payload.email))
        .one(&db)
        .await? else {
        return Err(UsersError::InvalidCredentials);
    };

    let parsed_password = PasswordHash::new(&user.password)?;

    if argon2
        .verify_password(payload.password.as_bytes(), &parsed_password)
        .is_err()
    {
        return Err(UsersError::InvalidCredentials);
    }

    let now_db = Utc::now();
    let now = OffsetDateTime::now_utc();

    let session = entity::sessions::Model {
        id: Uuid::now_v7(),
        user_id: user.id,
        created_at: now_db.naive_utc(),
        expires_at: (now_db + Duration::days(2)).naive_utc(),
    }
    .into_active_model()
    .insert(&db)
    .await?;

    #[allow(unused_mut)]
    let mut cookie = Cookie::build(SESSION_COOKIE_NAME, session.id.to_string())
        .path("/")
        .expires(now + time::Duration::days(2))
        .http_only(true);

    #[cfg(not(debug_assertions))]
    {
        cookie = cookie
            // TODO: use the actual musawarah domain
            .domain("salmanforgot.com")
            .secure(true);
    }

    #[cfg(debug_assertions)]
    {
        cookie = cookie.domain("localhost");
    }

    cookies.private(key).add(cookie.finish());

    Ok(())
}

/// User logout
#[utoipa::path(
    get,
    path = "/api/v1/users/logout",
    responses(
        (status = 200, description = "User logged out", body = UserToken),
    ),
    tag = "Users API"
)]
pub async fn logout(
    cookies: Cookies,
    State(db): State<DatabaseConnection>,
    auth: AuthExtractor,
) -> Result<(), UsersError> {
    let session = entity::sessions::ActiveModel {
        id: ActiveValue::Set(auth.session_id),
        ..Default::default()
    };
    session.delete(&db).await?;

    let mut cookie = Cookie::build(SESSION_COOKIE_NAME, "")
        .path("/")
        .http_only(true);

    #[cfg(not(debug_assertions))]
    {
        cookie = cookie
            // TODO: use the actual musawarah domain
            .domain("salmanforgot.com")
            .secure(true);
    }

    #[cfg(debug_assertions)]
    {
        cookie = cookie.domain("localhost");
    }

    cookies.remove(cookie.finish());

    Ok(())
}

/// Get user comics by username
#[utoipa::path(
    get,
    path = "/api/v1/users/comics/{username}",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "Caller authorized. returned requested user's comics", body = [ComicResponse]),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("auth" = [])
    ),
    tag = "Users API"
)]
#[axum::debug_handler]
pub async fn get_user_comics(
    State(db): State<DatabaseConnection>,
    Path(username): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<ComicResponseBrief>>, UsersError> {
    tracing::debug!("get {}'s comics", username);

    let Some(user) = entity::users::Entity::find()
        .filter(entity::users::Column::Username.eq(username))
        .one(&db)
        .await? else {
        return Err(UsersError::UserNotFound);
    };

    let comics = user
        .find_related(entity::comics::Entity)
        // .find_with_related(entity::chapters::Entity)
        .filter(entity::comics::Column::Id.gt(pagination.min_id))
        .filter(entity::comics::Column::Id.lt(pagination.max_id))
        // TODO: determine a good limit
        .limit(Some(10))
        .all(&db)
        .await?;

    let chapters = comics.load_many(entity::chapters::Entity, &db).await?;
    let users = comics.load_one(entity::users::Entity, &db).await?;

    let comics: Result<Vec<ComicResponseBrief>, UsersError> = multizip((comics, chapters, users))
        .map(|(comic, chapters, user)| {
            let user = user.ok_or(UsersError::InternalServerError)?;
            Ok(ComicResponseBrief {
                id: comic.id,
                author: UserResponseBrief {
                    id: user.id,
                    displayname: user.displayname,
                    username: user.username,
                    email: user.email,
                },
                title: comic.title,
                description: comic.description,
                created_at: comic.created_at.to_string(),
                chapters: chapters
                    .into_iter()
                    .map(|chapter| ChapterResponseBrief {
                        id: chapter.id,
                        description: chapter.description,
                        number: chapter.number,
                    })
                    .collect(),
            })
        })
        .collect();

    Ok(Json(comics?))
}

/// Get user by username
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    responses(
        (status = 200, description = "Caller authorized. returned current user info", body = UserClaims),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("auth" = [])
    ),
    tag = "Users API"
)]
#[axum::debug_handler]
pub async fn get_user(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>, UsersError> {
    let Some((user, Some(profile_image))) = entity::users::Entity::find()
        .filter(entity::users::Column::Id.eq(user_id))
        .find_also_related(entity::profile_images::Entity)
        .one(&db)
        // TODO: handle errors
        .await? else {
        tracing::debug!("User not found");
        return Err(UsersError::UserNotFound);
    };

    let user = UserResponse {
        id: user.id,
        displayname: user.displayname,
        username: user.username,
        email: user.email,
        profile_image: ImageResponse {
            content_type: profile_image.content_type,
            path: profile_image.path,
        },
    };

    Ok(Json(user))
}
