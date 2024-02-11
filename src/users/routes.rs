use std::sync::Arc;

use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use diesel::GroupedBy;
use diesel::{dsl::count, prelude::*};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use garde::Validate;
use itertools::multizip;
use itertools::Itertools;
use time::OffsetDateTime;
use tower_cookies::{cookie::Cookie, Cookies};
use uuid::Uuid;

use crate::{
    auth::AuthExtractor,
    coalesce,
    comics::comic_genres::models::{Genre, GenreMapping},
    comics::models::{Comic, ComicRating, ComicResponseBrief},
    common::models::ImageResponse,
    schema::comics,
    schema::{comic_chapters, comic_genres, profile_images, sessions, users},
    sessions::{
        models::{CreateSession, Session},
        SESSION_COOKIE_NAME,
    },
    users::models::User,
    utils::average_rating,
    AppState, InnerAppState,
};

use super::{
    email_verifications::routes::email_verification_router,
    models::{CreateUser, ProfileImage, UserLogin, UserResponse, UserResponseBrief, UserRole},
    UsersError,
};

pub fn users_router() -> Router<AppState> {
    Router::new()
        .route("/comics/:user_id", get(get_user_comics))
        .route("/logout", get(logout))
        .route("/:username", get(get_user))
        .route("/", post(create_user))
        .route("/login", post(login))
        .route("/me", get(me))
        .nest("/", email_verification_router())
}

/// get user by cookie
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    responses(
        (status = 200, description = "Caller authorized, returns user info", body = UserResponseBrief),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized"),
    ),
    tag = "Users API"
)]
pub async fn me(auth: AuthExtractor<{ UserRole::User as u32 }>) -> Json<UserResponseBrief> {
    Json(auth.current_user)
}

/// Create User
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body(
        content = CreateUser,
        description = "Validation:\n- username: 5-60\n- password: 8...",
        content_type = "application/json"),
    responses(
        (status = 200, description = "User successfully created", body = UserResponse),
        (status = StatusCode::BAD_REQUEST, description = "Fields validation error", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Users API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn create_user(
    State(state): State<Arc<InnerAppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<UserResponse>, UsersError> {
    payload.validate(&())?;
    if payload.username.is_empty() || payload.password.is_empty() || payload.email.is_empty() {
        return Err(UsersError::BadRequest);
    }

    let mut db = state.pool.get().await?;

    let salt = SaltString::generate(rand::thread_rng());

    // argon2 is a good algorithm (not a security expert :))
    let argon2 = Argon2::default();

    let hashed_password = argon2
        .hash_password(payload.password.as_bytes(), &salt)?
        .to_string();

    let (user, profile_image) = db
        .transaction::<_, UsersError, _>(|transaction| {
            async move {
                let user = User {
                    id: Uuid::now_v7(),
                    first_name: None,
                    last_name: None,
                    username: payload.username.to_lowercase(),
                    displayname: payload.username,
                    email: payload.email,
                    phone_number: None,
                    password: hashed_password,
                    bio: None,
                    role: UserRole::User,
                    created_at: Utc::now().naive_utc(),
                    updated_at: None,
                    last_login: None,
                };

                let user = diesel::insert_into(users::table)
                    .values(&user)
                    .returning(User::as_returning())
                    .get_result(transaction)
                    .await?;

                let profile_image = ProfileImage {
                    id: Uuid::now_v7(),
                    user_id: user.id,
                    path: String::from("ppL.webp"),
                    content_type: String::from("image/webp"),
                    updated_at: None,
                };

                let profile_image = diesel::insert_into(profile_images::table)
                    .values(&profile_image)
                    .returning(ProfileImage::as_returning())
                    .get_result(transaction)
                    .await?;

                Ok((user, profile_image))
            }
            .scope_boxed()
        })
        .await?;

    // TODO: handle error
    let bytes = state.storage.get_bytes(&profile_image.path).await.unwrap();

    Ok(Json(UserResponse {
        id: user.id,
        displayname: user.displayname,
        username: user.username,
        email: user.email,
        profile_image: ImageResponse {
            path: profile_image.path,
            content_type: profile_image.content_type,
            bytes,
        },
        role: user.role,
    }))
}

/// User login
#[utoipa::path(
    post,
    path = "/api/v1/users/login",
    request_body(
        content = UserLogin,
        description = "Validation:\n- password: min = 8",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "User authenticated"),
        (status = StatusCode::UNAUTHORIZED, description = "User unauthorized", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Users API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn login(
    State(state): State<Arc<InnerAppState>>,
    cookies: Cookies,
    Json(payload): Json<UserLogin>,
) -> Result<(), UsersError> {
    // TODO: add Result<Json<UserLogin>> and handle error

    payload.validate(&())?;

    let mut db = state.pool.get().await?;
    if let Some(session_id) = cookies
        .private(&state.cookies_secret)
        .get(SESSION_COOKIE_NAME)
    {
        if let Ok(session) = sessions::table
            .filter(
                sessions::id.eq(Uuid::parse_str(session_id.value())
                    .expect("encrypted valid uuid generated by server")),
            )
            .first::<Session>(&mut db)
            .await
        {
            tracing::error!("user already logged in with session id: {}", session.id);
            return Err(UsersError::AlreadyLoggedIn);
        }
    }

    // argon2 is a good algorithm (not a security expert :))
    let argon2 = Argon2::default();

    let user = users::table
        .filter(users::email.eq(&payload.email))
        .select(User::as_select())
        .first(&mut db)
        .await
        .map_err(|e| {
            if let diesel::result::Error::NotFound = e {
                return UsersError::InvalidCredentials;
            }
            UsersError::Diesel(e)
        })?;

    let parsed_password = PasswordHash::new(&user.password)?;

    if argon2
        .verify_password(payload.password.as_bytes(), &parsed_password)
        .is_err()
    {
        return Err(UsersError::InvalidCredentials);
    }

    let now = Utc::now();
    let time_now = OffsetDateTime::now_utc();

    let new_session = CreateSession {
        id: Uuid::now_v7(),
        user_id: user.id,
        created_at: now,
        expires_at: now + Duration::days(2),
    };

    let session = diesel::insert_into(sessions::table)
        .values(&new_session)
        .returning(Session::as_returning())
        .get_result::<Session>(&mut db)
        .await?;

    #[allow(unused_mut)]
    let mut cookie = Cookie::build(SESSION_COOKIE_NAME, session.id.to_string())
        .path("/")
        .expires(time_now + time::Duration::days(2))
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

    cookies.private(&state.cookies_secret).add(cookie.finish());

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
#[axum::debug_handler(state = AppState)]
pub async fn logout(
    cookies: Cookies,
    State(state): State<Arc<InnerAppState>>,
    auth: AuthExtractor<{ UserRole::User as u32 }>,
) -> Result<(), UsersError> {
    let mut db = state.pool.get().await?;

    diesel::delete(sessions::table.filter(sessions::id.eq(auth.session_id)))
        .execute(&mut db)
        .await?;

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

/// Get user comics by user id
#[utoipa::path(
    get,
    path = "/api/v1/users/comics/:user_id",
    responses(
        (status = 200, description = "Caller authorized. returned requested user's comics", body = [ComicResponseBrief]),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    security(
        ("auth" = [])
    ),
    tag = "Users API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_user_comics(
    State(state): State<Arc<InnerAppState>>,
    Path(user_id): Path<Uuid>,
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
) -> Result<Json<Vec<ComicResponseBrief>>, UsersError> {
    tracing::debug!("get {}'s comics", user_id);

    let mut db = state.pool.get().await?;

    let (comics, chapters_counts): (Vec<Comic>, Vec<i64>) = comics::table
        .filter(comics::user_id.eq(user_id))
        .left_join(comic_chapters::table)
        .group_by(comics::id)
        .select((
            Comic::as_select(),
            coalesce(count(comic_chapters::id).nullable(), 0),
        ))
        .load::<(Comic, i64)>(&mut db)
        .await?
        .into_iter()
        .multiunzip();

    let genres = GenreMapping::belonging_to(&comics)
        .inner_join(comic_genres::table)
        .select((GenreMapping::as_select(), Genre::as_select()))
        .load::<(GenreMapping, Genre)>(&mut db)
        .await?
        .grouped_by(&comics);

    let comics_ratings = ComicRating::belonging_to(&comics)
        .select(ComicRating::as_select())
        .load::<ComicRating>(&mut db)
        .await?
        .grouped_by(&comics);

    let comics: Result<Vec<ComicResponseBrief>, UsersError> =
        multizip((comics, genres, chapters_counts, comics_ratings))
            .map(|(comic, genres, chapters_count, comic_ratings)| {
                Ok(comic.into_response_brief(
                    genres.into_iter().map(|(_, genre)| genre).collect(),
                    chapters_count,
                    average_rating(comic_ratings),
                ))
            })
            .collect();

    Ok(Json(comics?))
}

/// Get user by username
#[utoipa::path(
    get,
    path = "/api/v1/users/:username",
    responses(
        (status = 200, description = "Caller authorized. returned current user info", body = UserClaims),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    security(
        ("auth" = [])
    ),
    tag = "Users API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_user(
    State(state): State<Arc<InnerAppState>>,
    Path(username): Path<String>,
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
) -> Result<Json<UserResponse>, UsersError> {
    let mut db = state.pool.get().await?;

    let user = users::table
        .filter(users::username.eq(username))
        .select(User::as_select())
        .first(&mut db)
        .await?;

    let profile_image = ProfileImage::belonging_to(&user)
        .select(ProfileImage::as_select())
        .first(&mut db)
        .await?;

    let bytes = state.storage.get_bytes(&profile_image.path).await.unwrap();

    let user = UserResponse {
        id: user.id,
        displayname: user.displayname,
        username: user.username,
        email: user.email,
        profile_image: ImageResponse {
            content_type: profile_image.content_type,
            path: profile_image.path,
            bytes,
        },
        role: user.role,
    };

    Ok(Json(user))
}
