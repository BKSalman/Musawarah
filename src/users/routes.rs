use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::GroupedBy;
use diesel_async::{
    pooled_connection::deadpool::Pool, scoped_futures::ScopedFutureExt, AsyncConnection,
    AsyncPgConnection, RunQueryDsl,
};
use garde::Validate;
use itertools::multizip;
use time::OffsetDateTime;
use tower_cookies::{cookie::Cookie, Cookies};
use uuid::Uuid;

use crate::{
    auth::AuthExtractor,
    chapters::models::{Chapter, ChapterResponseBrief},
    comic_genres::models::{ComicGenre, Genre, GenreMapping},
    comics::models::{Comic, ComicResponse},
    common::models::ImageResponse,
    schema::comics,
    schema::{comic_genres, profile_images, sessions, users},
    sessions::{
        models::{CreateSession, Session},
        SESSION_COOKIE_NAME,
    },
    users::models::User,
    AppState, PaginationParams, COOKIES_SECRET,
};

use super::{
    models::{CreateUser, ProfileImage, UserLogin, UserResponse, UserResponseBrief, UserRole},
    UsersError,
};

pub fn users_router() -> Router<AppState> {
    Router::new()
        .route("/comics/:username", get(get_user_comics))
        .route("/logout", get(logout))
        .route("/:user_id", get(get_user))
        .route("/", post(create_user))
        .route("/login", post(login))
}

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
#[axum::debug_handler(state = AppState)]
pub async fn create_user(
    State(state): State<AppState>,
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
                    // TODO: why do I need to clone this?
                    last_name: None,
                    username: payload.username.to_lowercase(),
                    displayname: payload.username.clone(),
                    email: payload.email.clone(),
                    phone_number: None,
                    password: hashed_password,
                    role: UserRole::User,
                    created_at: Utc::now().naive_utc(),
                    updated_at: None,
                    last_login: None,
                };
                // .into_active_model()
                // .insert(&state.db)
                // .await
                // .map_err(|err| {
                //     // tracing::error!("error {:#?}", err);
                //     if let migration::DbErr::Query(sea_orm::RuntimeErr::SqlxError(sqlx::Error::Database(
                //         error,
                //     ))) = err
                //     {
                //         return match error.constraint() {
                //             Some("users_username_key") => {
                //                 tracing::error!("{}", error);
                //                 UsersError::Conflict(String::from("username already taken"))
                //             }
                //             Some("users_email_key") => {
                //                 tracing::error!("{}", error);
                //                 UsersError::Conflict(String::from("email already taken"))
                //             }
                //             _ => {
                //                 tracing::error!("sqlx error: {}", error);
                //                 UsersError::InternalServerError
                //             }
                //         };
                //     }
                //     tracing::error!("Db error: {}", err);
                //     UsersError::InternalServerError
                // })?;

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

                return Ok((user, profile_image));
            }
            .scope_boxed()
        })
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
#[axum::debug_handler(state = AppState)]
pub async fn login(
    State(pool): State<Pool<AsyncPgConnection>>,
    cookies: Cookies,
    Json(payload): Json<UserLogin>,
) -> Result<(), UsersError> {
    payload.validate(&())?;

    let mut db = pool.get().await?;

    let key = COOKIES_SECRET.get().expect("cookies secret key");

    if let Some(session_id) = cookies.private(key).get(SESSION_COOKIE_NAME) {
        tracing::error!("user already logged in with session id: {session_id}");
        return Err(UsersError::AlreadyLoggedIn);
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
            return UsersError::Diesel(e);
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
#[axum::debug_handler(state = AppState)]
pub async fn logout(
    cookies: Cookies,
    State(pool): State<Pool<AsyncPgConnection>>,
    auth: AuthExtractor<{ UserRole::User as u32 }>,
) -> Result<(), UsersError> {
    let mut db = pool.get().await?;

    diesel::delete(sessions::table.filter(sessions::id.eq(auth.session_id))).execute(&mut db);

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
#[axum::debug_handler(state = AppState)]
pub async fn get_user_comics(
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(username): Path<String>,
    Query(pagination): Query<PaginationParams>,
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
) -> Result<Json<Vec<ComicResponse>>, UsersError> {
    tracing::debug!("get {}'s comics", username);

    let mut db = pool.get().await?;

    let user = users::table
        .filter(users::username.eq(username))
        .select(User::as_select())
        .first(&mut db)
        .await
        .map_err(|e| {
            if let diesel::result::Error::NotFound = e {
                return UsersError::UserNotFound;
            }
            e.into()
        })?;

    // TODO: this can by inner_join'ed
    let comics: Vec<Comic> = Comic::belonging_to(&user)
        .filter(comics::id.gt(pagination.min_id))
        .filter(comics::id.lt(pagination.max_id))
        .limit(10)
        .select(Comic::as_select())
        .load::<Comic>(&mut db)
        .await?;

    let chapters: Vec<Chapter> = Chapter::belonging_to(&comics)
        .select(Chapter::as_select())
        .load::<Chapter>(&mut db)
        .await?;

    let genres: Vec<(GenreMapping, Genre)> = GenreMapping::belonging_to(&comics)
        .inner_join(comic_genres::table)
        .select((GenreMapping::as_select(), Genre::as_select()))
        .load::<(GenreMapping, Genre)>(&mut db)
        .await?;

    let chapters = chapters.grouped_by(&comics);

    let genres = genres.grouped_by(&comics);

    let comics: Result<Vec<ComicResponse>, UsersError> = multizip((comics, genres, chapters))
        .map(move |(comic, genres, chapters)| {
            Ok(ComicResponse {
                id: comic.id,
                author: UserResponseBrief {
                    id: user.id,
                    displayname: user.displayname.clone(),
                    username: user.username.clone(),
                    email: user.email.clone(),
                    role: user.role,
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
                genres: genres
                    .into_iter()
                    .map(|(_genre_mapping, genre)| ComicGenre {
                        id: genre.id,
                        name: genre.name,
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
#[axum::debug_handler(state = AppState)]
pub async fn get_user(
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(user_id): Path<Uuid>,
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
) -> Result<Json<UserResponse>, UsersError> {
    let mut db = pool.get().await?;

    let user = users::table
        .filter(users::id.eq(user_id))
        .select(User::as_select())
        .first(&mut db)
        .await?;

    let profile_image = ProfileImage::belonging_to(&user)
        .select(ProfileImage::as_select())
        .first(&mut db)
        .await?;

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
