use std::io::SeekFrom;

use axum::{
    extract::{Multipart, Path, Query, State},
    Json,
};
use futures::TryStreamExt;
use sqlx::PgPool;
use tempfile::tempfile;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{
    s3::{interface::Storage, Upload},
    users::models::{UserClaims, UserResponse},
    PaginationParams,
};

use super::{
    models::{ImageResponse, PostData, PostResponse},
    utils::box_error,
    PostsError,
};

const ALLOWED_MIME_TYPES: [&str; 3] = ["image/jpeg", "image/jpg", "image/png"];

/// Create Post
#[utoipa::path(
    post,
    path = "/api/posts/",
    request_body(content = CreatePost, description = "something something", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Create new post", body = PostResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Posts API"
)]
pub async fn create_post(
    user_claims: UserClaims,
    State(storage): State<Storage>,
    State(db): State<PgPool>,
    mut fields: Multipart,
) -> Result<Json<PostResponse>, PostsError> {
    let mut post = PostData::builder();
    let mut upload = Upload::builder();
    let mut content_length: i64 = 0;

    while let Some(mut field) = fields.next_field().await.map_err(|err| {
        tracing::debug!("create_post mutipart error: {:#?}", err);
        PostsError::InternalServerError
    })? {
        if let Some(field_name) = field.name() {
            match field_name {
                "title" => {
                    tracing::debug!("adding title");
                    post = post.title(field.text().await.map_err(|err| {
                        tracing::debug!("title field error: {:#?}", err);
                        PostsError::InternalServerError
                    })?)
                }
                "content" => {
                    tracing::debug!("adding content");
                    post = post.content(field.text().await.map_err(|err| {
                        tracing::debug!("title field error: {:#?}", err);
                        PostsError::InternalServerError
                    })?)
                }
                "image" => {
                    tracing::debug!("adding image");
                    if !ALLOWED_MIME_TYPES
                        .contains(&field.content_type().ok_or(PostsError::BadRequest)?)
                    {
                        return Err(PostsError::BadRequest);
                    }

                    let file_name = field.file_name().ok_or(PostsError::BadRequest)?.to_string();

                    let temp_file = tempfile().expect("temp file");
                    let mut temp_file = File::from_std(temp_file);

                    while let Some(chunk) = field.chunk().await.map_err(|err| {
                        tracing::debug!("image field chunk error: {:#?}", err);
                        PostsError::BadRequest
                    })? {
                        content_length += chunk.len() as i64;
                        if let Err(err) = temp_file.write_all(&chunk).await {
                            tracing::debug!("tempfile write error: {:#?}", err);
                            return Err(PostsError::InternalServerError);
                        }
                    }

                    temp_file.seek(SeekFrom::Start(0)).await.expect("seek file");

                    let s3_file_name = format!("{}_{}", file_name, Uuid::now_v7());

                    upload = upload
                        .path(s3_file_name)
                        .stream(ReaderStream::new(temp_file).map_err(|err| {
                            tracing::debug!("tempfile stream error: {:#?}", err);
                            box_error(PostsError::InternalServerError)
                        }))
                        .content_type(
                            field
                                .content_type()
                                .ok_or_else(|| {
                                    tracing::debug!("image field no content_type");
                                    PostsError::InternalServerError
                                })?
                                .to_string(),
                        );
                }
                _ => continue,
            }
        }
    }

    let post = post.build().map_err(|_| PostsError::BadRequest)?;

    let upload = upload.build().map_err(|_| PostsError::BadRequest)?;

    let mut transaction = db.begin().await?;

    // save post to db
    let post = sqlx::query!(
        r#"
INSERT INTO posts ( id, author_id, title, content)
VALUES ( $1, $2, $3, $4 )

RETURNING *
            "#,
        Uuid::now_v7(),
        user_claims.user.id,
        post.title,
        post.content,
    )
    .fetch_one(&mut *transaction)
    .await?;

    // save to image to db
    let image = sqlx::query!(
        r#"
INSERT INTO images ( id, user_id, post_id, path, content_type )
VALUES ( $1, $2, $3, $4 , $5 )

RETURNING *
    "#,
        Uuid::now_v7(),
        user_claims.user.id,
        post.id,
        upload.path,
        upload.content_type,
    )
    .fetch_one(&mut *transaction)
    .await?;

    // upload image to s3
    if let Err(err) = storage
        .put(&upload.path, upload.stream, content_length)
        .await
        .map_err(|err| {
            tracing::debug!("s3 error: {:#?}", err);
            PostsError::InternalServerError
        })
    {
        tracing::debug!("failed to upload to s3 rolling back");
        transaction.rollback().await?;
        return Err(err);
    }

    transaction.commit().await?;
    let post = PostResponse {
        id: post.id,
        title: post.title,
        content: post.content,
        created_at: post.created_at.to_string(),
        user: UserResponse {
            id: user_claims.user.id,
            username: user_claims.user.username,
            email: user_claims.user.email,
        },
        image: ImageResponse {
            content_type: image.content_type,
            path: image.path,
        },
    };

    Ok(Json(post))
}

/// Get post by id
#[utoipa::path(
    get,
    path = "/api/posts/{post_id}",
    params(
        ("post_id" = Uuid, Path, description = "ID of the requested post"),
    ),
    responses(
        (status = 200, description = "Caller authorized. returned requested post", body = PostResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Posts API"
)]
pub async fn get_post(
    State(db): State<PgPool>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<PostResponse>, PostsError> {
    let Some(record) = sqlx::query!(
        r#"
SELECT images.id AS image_id, posts.id AS post_id, users.id AS user_id, images.content_type,
images.path, posts.title, posts.content, posts.created_at,
users.username, users.email

FROM posts
INNER JOIN images
ON posts.id = images.post_id
INNER JOIN users
ON posts.author_id = users.id
WHERE posts.id = $1
        "#,
        post_id
    )
    .fetch_optional(&db)
    // TODO: map this error
    .await? else {
        return Err(PostsError::PostNotFound);
    };

    let post = PostResponse {
        id: record.post_id,
        title: record.title,
        content: record.content,
        created_at: record.created_at.to_string(),
        user: UserResponse {
            id: record.user_id,
            username: record.username,
            email: record.email,
        },
        image: ImageResponse {
            content_type: record.content_type,
            path: record.path,
        },
    };

    Ok(Json(post))
}

/// Get posts with pagination
#[utoipa::path(
    get,
    path = "/api/posts/",
    params(
        PaginationParams,
    ),
    responses(
        (status = 200, description = "", body = [PostResponse]),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Posts API"
)]
pub async fn get_posts_cursor(
    State(db): State<PgPool>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<PostResponse>>, PostsError> {
    tracing::debug!("cursor: {:#?}", pagination);
    let records = sqlx::query!(
        r#"
SELECT images.id AS image_id, posts.id AS post_id, users.id AS user_id, images.content_type,
images.path, posts.title, posts.content, posts.created_at,
users.username, users.email

FROM posts
INNER JOIN images
ON posts.id = images.post_id
INNER JOIN users
ON posts.author_id = users.id

WHERE posts.id > $1 AND posts.id < $2
ORDER BY posts.id DESC

LIMIT 10
        "#,
        pagination.min_id,
        pagination.max_id,
    )
    .fetch_all(&db)
    // TODO: map this error
    .await?;

    // first element in the vector is the newest
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

    Ok(Json(posts))
}
