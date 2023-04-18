use std::io::SeekFrom;

use axum::{
    extract::{Multipart, State},
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

use crate::{
    chapters::utils::box_error,
    s3::{interface::Storage, Upload},
    users::models::UserClaims,
};

use super::{
    models::{ChapterData, ChapterResponse},
    ChaptersError,
};

use uuid::Uuid;

const ALLOWED_MIME_TYPES: [&str; 3] = ["image/jpeg", "image/jpg", "image/png"];

pub async fn create_chapter(
    user_claims: UserClaims,
    State(storage): State<Storage>,
    State(db): State<PgPool>,
    mut fields: Multipart,
) -> Result<Json<ChapterResponse>, ChaptersError> {
    let mut chapter = ChapterData::builder();
    let mut upload = Upload::builder();
    let mut content_length: i64 = 0;

    while let Some(mut field) = fields.next_field().await.map_err(|err| {
        tracing::debug!("create_chapter mutipart error: {:#?}", err);
        ChaptersError::InternalServerError
    })? {
        if let Some(field_name) = field.name() {
            match field_name {
                "title" => {
                    tracing::debug!("adding title");
                    chapter = chapter.title(field.text().await.map_err(|err| {
                        tracing::debug!("title field error: {:#?}", err);
                        ChaptersError::InternalServerError
                    })?)
                }
                "description" => {
                    tracing::debug!("adding description");
                    chapter = chapter.description(field.text().await.map_err(|err| {
                        tracing::debug!("title field error: {:#?}", err);
                        ChaptersError::InternalServerError
                    })?)
                }
                "image" => {
                    tracing::debug!("adding image");
                    if !ALLOWED_MIME_TYPES
                        .contains(&field.content_type().ok_or(ChaptersError::BadRequest)?)
                    {
                        return Err(ChaptersError::BadRequest);
                    }

                    let file_name = field
                        .file_name()
                        .ok_or(ChaptersError::BadRequest)?
                        .to_string();

                    let temp_file = tempfile().expect("temp file");
                    let mut temp_file = File::from_std(temp_file);

                    while let Some(chunk) = field.chunk().await.map_err(|err| {
                        tracing::debug!("image field chunk error: {:#?}", err);
                        ChaptersError::BadRequest
                    })? {
                        content_length += chunk.len() as i64;
                        if let Err(err) = temp_file.write_all(&chunk).await {
                            tracing::debug!("tempfile write error: {:#?}", err);
                            return Err(ChaptersError::InternalServerError);
                        }
                    }

                    temp_file.seek(SeekFrom::Start(0)).await.expect("seek file");

                    let s3_file_name = format!("{}_{}", file_name, Uuid::now_v7());

                    upload = upload
                        .path(s3_file_name)
                        .stream(ReaderStream::new(temp_file).map_err(|err| {
                            tracing::debug!("tempfile stream error: {:#?}", err);
                            box_error(ChaptersError::InternalServerError)
                        }))
                        .content_type(
                            field
                                .content_type()
                                .ok_or_else(|| {
                                    tracing::debug!("image field no content_type");
                                    ChaptersError::InternalServerError
                                })?
                                .to_string(),
                        );
                }
                _ => continue,
            }
        }
    }

    let chapter = chapter.build().map_err(|_| ChaptersError::BadRequest)?;

    let upload = upload.build().map_err(|_| ChaptersError::BadRequest)?;

    let mut transaction = db.begin().await?;

    // save chapter to db
    let chapter = sqlx::query!(
        r#"
INSERT INTO comics ( id, author_id, title, description)
VALUES ( $1, $2, $3, $4 )

RETURNING *
            "#,
        Uuid::now_v7(),
        user_claims.user.id,
        chapter.title,
        chapter.description,
    )
    .fetch_one(&mut *transaction)
    .await?;

    // save to image to db
    let image = sqlx::query!(
        r#"
INSERT INTO images ( id, user_id, comic_id, path, content_type )
VALUES ( $1, $2, $3, $4 , $5 )

RETURNING *
    "#,
        Uuid::now_v7(),
        user_claims.user.id,
        chapter.id,
        upload.path,
        upload.content_type,
    )
    .fetch_one(&mut *transaction)
    .await?;

    // upload image to s3
    if let Err(err) = storage
        .put(
            &upload.path,
            upload.stream,
            content_length,
            &upload.content_type,
        )
        .await
        .map_err(|err| {
            tracing::debug!("s3 error: {:#?}", err);
            ChaptersError::InternalServerError
        })
    {
        tracing::debug!("failed to upload to s3 rolling back");
        transaction.rollback().await?;
        return Err(err);
    }

    transaction.commit().await?;
    let chapter = ChapterResponse {
        number: 1,
        description: todo!(),
        pages: todo!(),
    };

    Ok(Json(chapter))
}
