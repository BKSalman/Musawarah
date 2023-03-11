use std::io::SeekFrom;

use axum::{
    extract::{Multipart, Path, State},
    Json,
};
use futures::TryStreamExt;
use serde_json::{json, Value};
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
    users::models::{UserClaims, UserReponse},
};

use super::{
    models::{CreatePost, ImageResponse, PostResponse},
    utils::box_error,
    PostsError,
};

const ALLOWED_MIME_TYPES: [&str; 3] = ["image/jpeg", "image/jpg", "image/png"];

pub async fn create_post(
    user_claims: UserClaims,
    State(storage): State<Storage>,
    State(db): State<PgPool>,
    mut fields: Multipart,
) -> Result<Json<Value>, PostsError> {
    let mut post = CreatePost::builder();
    let mut upload = Upload::builder();
    let mut content_length: i64 = 0;

    while let Some(mut field) = fields
        .next_field()
        .await
        .map_err(|_| PostsError::InternalServerError)?
    {
        if let Some(field_name) = field.name() {
            match field_name {
                "title" => {
                    tracing::debug!("adding title");
                    post = post.title(
                        field
                            .text()
                            .await
                            .map_err(|_| PostsError::InternalServerError)?,
                    )
                }
                "content" => {
                    tracing::debug!("adding content");
                    post = post.content(
                        field
                            .text()
                            .await
                            .map_err(|_| PostsError::InternalServerError)?,
                    )
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

                    while let Some(chunk) = field
                        .chunk()
                        .await
                        .map_err(|_| PostsError::InternalServerError)?
                    {
                        content_length += chunk.len() as i64;
                        if let Err(_err) = temp_file.write_all(&chunk).await {
                            return Err(PostsError::InternalServerError);
                        }
                    }

                    temp_file.seek(SeekFrom::Start(0)).await.expect("seek file");

                    let s3_file_name = format!("{}_{}", file_name, Uuid::now_v7());

                    upload = upload
                        .path(s3_file_name)
                        .stream(
                            ReaderStream::new(temp_file)
                                .map_err(|_| box_error(PostsError::InternalServerError)),
                        )
                        .content_type(
                            field
                                .content_type()
                                .ok_or(PostsError::InternalServerError)?
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

    let user_id =
        Uuid::parse_str(&user_claims.user.id).map_err(|_| PostsError::InternalServerError)?;

    // save post to db
    let post_id = sqlx::query!(
        r#"
INSERT INTO posts ( id, author_id, title, content)
VALUES ( $1, $2, $3, $4 )
RETURNING id
            "#,
        Uuid::now_v7(),
        user_id,
        post.title,
        post.content,
    )
    .fetch_one(&mut *transaction)
    .await?
    .id;

    // save to image to db
    let id = sqlx::query!(
        r#"
INSERT INTO images ( id, user_id, post_id, path, content_type )
VALUES ( $1, $2, $3, $4 , $5 )
RETURNING id
    "#,
        Uuid::now_v7(),
        user_id,
        post_id,
        upload.path,
        upload.content_type,
    )
    .fetch_one(&mut *transaction)
    .await?
    .id;

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

    Ok(Json(json!({
        "post_id": id.to_string(),
    })))
}

pub async fn get_post(
    // State(storage): State<Storage>,
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
        id: record.post_id.to_string(),
        title: record.title,
        content: record.content,
        created_at: record.created_at.to_string(),
        user: UserReponse {
            id: record.user_id.to_string(),
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

pub async fn get_posts_cursor(
    State(db): State<PgPool>,
    Path(cursor): Path<Uuid>,
) -> Result<Json<Vec<PostResponse>>, PostsError> {
    tracing::debug!("cursor: {}", cursor);
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

WHERE posts.id < $1
ORDER BY posts.id

LIMIT 10
        "#,
        cursor
    )
    .fetch_all(&db)
    // TODO: map this error
    .await?;

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

pub async fn get_posts(State(db): State<PgPool>) -> Result<Json<Vec<PostResponse>>, PostsError> {
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

ORDER BY posts.id

LIMIT 10
        "#,
    )
    .fetch_all(&db)
    // TODO: map this error
    .await?;

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
