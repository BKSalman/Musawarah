use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use garde::Validate;
use itertools::multizip;
use jwt_simple::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, LoaderTrait,
    ModelTrait, QueryFilter, QuerySelect,
};
use uuid::Uuid;

use crate::{
    chapters::models::ChapterResponseBrief,
    comics::models::{ComicResponseBrief, ImageResponse},
    entity, AppState, PaginationParams, JWT_KEY,
};

use super::{
    models::{CreateUser, UserClaims, UserLogin, UserResponse, UserResponseBrief, UserToken},
    UsersError,
};

/// Create User
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body(content = CreateUser, description = "Username, Email, and password", content_type = "application/json"),
    responses(
        (status = 200, description = "User successfully created", body = UserResponse),
        (status = StatusCode::BAD_REQUEST, description = "Fields validation error", body = ErrorHandlingResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Users API"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<Uuid>, UsersError> {
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
    }
    .into_active_model()
    .insert(&state.db)
    .await?;

    let _profile_image = entity::profile_images::Model {
        id: Uuid::now_v7(),
        user_id: user.id,
        path: String::from("ppL.webp"),
        content_type: String::from("image/webp"),
    }
    .into_active_model()
    .insert(&state.db)
    .await?;

    //     .map_err(|e| match e {
    //         sqlx::Error::Database(dbe) => match dbe.constraint() {
    //             Some("users_username_key") => UsersError::Conflict("username taken".into()),
    //             Some("users_email_key") => UsersError::Conflict("email taken".into()),
    //             _ => {
    //                 tracing::debug!("create_user db error: {:#?}", dbe);
    //                 UsersError::InternalServerError
    //             }
    //         },
    //         _ => {
    //             // TODO: log this instead of printing
    //             tracing::debug!("{e:#?}");
    //             UsersError::InternalServerError
    //         }
    //     })?;

    Ok(Json(user.id))
}

/// User login
#[utoipa::path(
    post,
    path = "/api/v1/users/login",
    request_body(content = UserLogin, description = "Email and password", content_type = "application/json"),
    responses(
        (status = 200, description = "User authenticated", body = UserToken),
        (status = StatusCode::UNAUTHORIZED, description = "User unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Users API"
)]
pub async fn login(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UserLogin>,
) -> Result<Json<UserToken>, UsersError> {
    payload.validate(&())?;
    // argon2 is a good algorithm (not a security expert :))
    let argon2 = Argon2::default();

    //     let record = sqlx::query!(
    //         r#"
    // SELECT users.id AS user_id, users.displayname, users.username, users.email, users.password,
    // profile_images.path, profile_images.content_type
    // FROM users

    // INNER JOIN profile_images
    // ON users.id = profile_images.user_id

    // WHERE email = $1;
    //         "#,
    //         payload.email,
    //     )
    //     .fetch_optional(&db)
    //     .await
    //     // TODO: better error handling
    //     .map_err(|error| match error {
    //         _ => {
    //             tracing::debug!("login db error: {:#?}", error);
    //             UsersError::InternalServerError
    //         }
    //     })?;

    // let Some(record) = record else {
    //     return Err(UsersError::UserNotFound);
    // };

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

    // TODO: profile image
    let Some(profile_image) = user
        .find_related(entity::profile_images::Entity)
        .one(&db)
        .await? else {
        return Err(UsersError::InternalServerError);
    };

    let claims = Claims::with_custom_claims(
        UserClaims {
            user: UserResponse {
                id: user.id,
                displayname: user.displayname,
                username: user.username,
                email: user.email,
                profile_image: ImageResponse {
                    path: profile_image.path,
                    content_type: profile_image.content_type,
                },
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

/// Get user comics by username
#[utoipa::path(
    get,
    path = "/api/v1/users/comics/{username}",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "Caller authorized. returned requested user's comics", body = [PostResponse]),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users API"
)]
pub async fn get_user_comics(
    // prevent non logged users from
    // accessing a specific user's posts
    _: UserClaims,
    State(db): State<DatabaseConnection>,
    Path(username): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<ComicResponseBrief>>, UsersError> {
    let Some(user) = entity::users::Entity::find()
        .filter(entity::users::Column::Username.eq(username))
        .one(&db)
        .await? else {
        return Err(UsersError::UserNotFound);
    };

    // FIXME: query comics, chapters, and chapter pages in same query
    // or 2 separate queries

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
        ("jwt" = [])
    ),
    tag = "Users API"
)]
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
