use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    chapters::models::{ChapterPageResponse, ChapterResponse},
    users::models::{UserClaims, UserResponse},
    PaginationParams,
};

use super::{
    models::{ComicResponse, CreateComic, ImageResponse},
    ComicsError,
};

/// Create Comic
#[utoipa::path(
    post,
    path = "/api/v1/comics/",
    request_body(content = CreateComic, content_type = "application/json"),
    responses(
        (status = 200, description = "Create new comic", body = ComicResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Comics API"
)]
pub async fn create_comic(
    user_claims: UserClaims,
    State(db): State<PgPool>,
    Json(payload): Json<CreateComic>,
) -> Result<Json<ComicResponse>, ComicsError> {
    // save comic to db
    let comic = sqlx::query!(
        r#"
INSERT INTO comics ( id, author_id, title, description)
VALUES ( $1, $2, $3, $4 )

RETURNING *
            "#,
        Uuid::now_v7(),
        user_claims.user.id,
        payload.title,
        payload.description,
    )
    .fetch_one(&db)
    .await?;

    let comic = ComicResponse {
        id: comic.id,
        title: comic.title,
        description: comic.description,
        created_at: comic.created_at.to_string(),
        author: UserResponse {
            id: user_claims.user.id,
            username: user_claims.user.username,
            displayname: user_claims.user.displayname,
            email: user_claims.user.email,
            profile_image: user_claims.user.profile_image,
        },
        chapters: vec![],
    };

    Ok(Json(comic))
}

#[derive(Deserialize)]
pub struct GetComicParams {
    comic_id: Uuid,
}

/// Get comic by id
#[utoipa::path(
    get,
    path = "/api/v1/comics/{comic_id}",
    params(
        ("comic_id" = Uuid, Path, description = "ID of the requested comic"),
    ),
    responses(
        (status = 200, description = "Caller authorized. returned requested comic", body = ComicResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Comics API"
)]
pub async fn get_comic(
    State(db): State<PgPool>,
    Path(params): Path<GetComicParams>,
) -> Result<Json<Vec<ComicResponse>>, ComicsError> {
    let records = sqlx::query!(
        r#"
SELECT comics.id AS comic_id, users.id AS user_id,
comics.title, comics.description AS comic_desc, comics.created_at,
users.username, users.email, users.displayname,
profile_images.path AS profile_image_path, profile_images.content_type AS profile_image_content_type,
chapters.number AS chapter_num, chapters.description AS chapter_desc,
chapter_pages.number AS chapter_page_num

FROM comics
INNER JOIN images
ON comics.id = images.comic_id
INNER JOIN users
ON comics.author_id = users.id
INNER JOIN profile_images
ON comics.author_id = profile_images.user_id
INNER JOIN chapters
ON comics.author_id = chapters.author_id
INNER JOIN chapter_pages
ON comics.author_id = chapter_pages.author_id
WHERE comics.id = $1
LIMIT 10
        "#,
        params.comic_id,
    ).map(|row| {

            ComicResponse {
        id: row.comic_id,
        title: row.title,
        description: row.comic_desc,
        created_at: row.created_at.to_string(),
        author: UserResponse {
            id: row.user_id,
            username: row.username,
            displayname: row.displayname,
            email: row.email,
            profile_image: ImageResponse {
                path: row.profile_image_path,
                content_type: row.profile_image_content_type,
            },
        },
        chapters: vec![ChapterResponse {
            number: row.chapter_num,
            description: row.chapter_desc,
            pages: vec![ChapterPageResponse {
                number: row.chapter_page_num,
                image: ImageResponse {
                    content_type: String::new(),
                    path: String::new(),
                },
            }],
        }],
    }
        })
    .fetch_all(&db)
    // TODO: map this error
    .await?;

    Ok(Json(records))
}

/// Get comics with pagination
#[utoipa::path(
    get,
    path = "/api/v1/comics/",
    params(
        PaginationParams,
    ),
    responses(
        (status = 200, description = "", body = [ComicResponse]),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Comics API"
)]
pub async fn get_comics_cursor(
    State(db): State<PgPool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<ComicResponse>>, ComicsError> {
    tracing::debug!("cursor: {:#?}", pagination);
    let records = sqlx::query!(
        r#"
SELECT images.id AS image_id, comics.id AS comic_id, users.id AS user_id, images.content_type,
images.path, comics.title, comics.description, comics.created_at,
users.username, users.email, users.displayname,
profile_images.path AS profile_image_path, profile_images.content_type AS profile_image_content_type

FROM comics
INNER JOIN images
ON comics.id = images.comic_id
INNER JOIN users
ON comics.author_id = users.id
INNER JOIN profile_images
ON comics.author_id = profile_images.user_id

WHERE comics.id > $1 AND comics.id < $2
ORDER BY comics.id DESC

LIMIT 10
        "#,
        pagination.min_id,
        pagination.max_id,
    )
    .fetch_all(&db)
    // TODO: map this error
    .await?;

    // first element in the vector is the newest
    let comics = records
        .into_iter()
        .map(|r| ComicResponse {
            id: r.comic_id,
            title: r.title,
            description: r.description,
            created_at: r.created_at.to_string(),
            author: UserResponse {
                id: r.user_id,
                username: r.username,
                displayname: r.displayname,
                email: r.email,
                profile_image: ImageResponse {
                    path: r.profile_image_path,
                    content_type: r.profile_image_content_type,
                },
            },
            chapters: vec![],
        })
        .collect::<Vec<ComicResponse>>();

    Ok(Json(comics))
}
