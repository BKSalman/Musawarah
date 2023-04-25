use std::io::SeekFrom;

use axum::{
    extract::{Multipart, Path, Query, State},
    Json,
};
use chrono::Utc;
use futures::{FutureExt, TryStreamExt};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QuerySelect, Set, TransactionTrait,
};
use serde_json::json;
use tempfile::tempfile;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};
use tokio_util::io::ReaderStream;

use crate::{
    comics::models::ImageResponse,
    entity::{self, chapter_pages::Entity as ChapterPage, chapters::Entity as Chapter},
    s3::{interface::Storage, Upload},
    users::models::UserClaims,
    PaginationParams,
};

use super::{
    models::{
        ChapterPageData, ChapterPageResponse, ChapterResponse, ChapterResponseBrief, CreateChapter,
        UpdateChapter,
    },
    utils::box_error,
    ChaptersError,
};

use uuid::Uuid;

const ALLOWED_MIME_TYPES: [&str; 3] = ["image/jpeg", "image/jpg", "image/png"];

/// Create a chapter
#[utoipa::path(
    post,
    path = "/api/v1/chapters",
    request_body(content = CreateChapter, content_type = "application/json"),
    responses(
        (status = 200, description = "Comic successfully created", body = UserResponse),
        (status = StatusCode::CONFLICT, description = "Chapter number conflicts with an already existing one", body = ErrorHandlingResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Chapters API"
)]
pub async fn create_chapter(
    user_claims: UserClaims,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateChapter>,
) -> Result<Json<ChapterResponseBrief>, ChaptersError> {
    let current_date = Utc::now().naive_utc();
    let chapter = entity::chapters::Model {
        id: Uuid::now_v7(),
        author_id: user_claims.user.id,
        comic_id: payload.comic_id,
        number: payload.number,
        title: payload.title,
        description: payload.description,
        created_at: current_date,
        updated_at: current_date,
    }
    .into_active_model()
    .insert(&db)
    .await
    .map_err(|err| {
        // tracing::debug!("error {:#?}", err);
        match err {
            migration::DbErr::Query(e) => match e {
                sea_orm::RuntimeErr::SqlxError(sqlx_err) => match sqlx_err {
                    sqlx::Error::Database(err) => match err.constraint() {
                        Some("chapters_title_key") => {
                            tracing::error!("{}", err);
                            ChaptersError::Conflict(String::from(
                                "chapter with same title already exists",
                            ))
                        }
                        Some("chapters_comic_id_number_key") => {
                            tracing::error!("{}", err);
                            ChaptersError::Conflict(String::from(
                                "chapter with same number already exists",
                            ))
                        }
                        _ => {
                            tracing::error!("{}", err);
                            ChaptersError::InternalServerError
                        }
                    },
                    _ => {
                        tracing::error!("{}", sqlx_err);
                        ChaptersError::InternalServerError
                    }
                },
                sea_orm::RuntimeErr::Internal(internal_err) => {
                    tracing::error!("DB internal error: {}", internal_err);
                    ChaptersError::InternalServerError
                }
            },
            _ => {
                tracing::error!("DB error: {}", err);
                ChaptersError::InternalServerError
            }
        }
    })?;

    Ok(Json(chapter.into()))
}

/// Create a chapter page
#[utoipa::path(
    post,
    path = "/api/v1/chapters/page",
    request_body(content = CreateChapterPage, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Chapter page successfully created", body = ChapterResponse),
        (status = StatusCode::BAD_REQUEST, description = "Fields validation error", body = ErrorHandlingResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Chapters API"
)]
pub async fn create_chapter_page(
    user_claims: UserClaims,
    State(storage): State<Storage>,
    State(db): State<DatabaseConnection>,
    mut fields: Multipart,
) -> Result<Json<ChapterPageResponse>, ChaptersError> {
    let mut chapter_page = ChapterPageData::builder();
    let mut upload = Upload::builder();
    let mut content_length: i64 = 0;

    while let Some(mut field) = fields.next_field().await.map_err(|err| {
        tracing::debug!("create_chapter mutipart error: {:#?}", err);
        ChaptersError::InternalServerError
    })? {
        if let Some(field_name) = field.name() {
            match field_name {
                "description" => {
                    tracing::debug!("adding description");
                    chapter_page = chapter_page.description(field.text().await.ok());
                }
                "number" => {
                    tracing::debug!("adding chapter number");
                    chapter_page = chapter_page.number(
                        field
                            .text()
                            .await
                            .map_err(|e| {
                                tracing::error!("number field error: {:#?}", e);
                                ChaptersError::BadRequest
                            })?
                            .parse()
                            .map_err(|e| {
                                tracing::error!("number field error: {:#?}", e);
                                ChaptersError::BadRequest
                            })?,
                    );
                }
                "comic_id" => {
                    tracing::debug!("adding comic_id");

                    chapter_page = chapter_page.comic_id(
                        Uuid::parse_str(&field.text().await.map_err(|err| {
                            tracing::error!("comic_id field error: {:#?}", err);
                            ChaptersError::BadRequest
                        })?)
                        .map_err(|e| {
                            tracing::error!("comic_id field error: {:#?}", e);
                            ChaptersError::BadRequest
                        })?,
                    );
                }
                "chapter_id" => {
                    tracing::debug!("adding chapter_id");

                    chapter_page = chapter_page.chapter_id(
                        Uuid::parse_str(&field.text().await.map_err(|err| {
                            tracing::error!("chapter_id field error: {:#?}", err);
                            ChaptersError::BadRequest
                        })?)
                        .map_err(|e| {
                            tracing::error!("chapter_id field error: {:#?}", e);
                            ChaptersError::BadRequest
                        })?,
                    );
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
                        tracing::error!("image field chunk error: {:#?}", err);
                        ChaptersError::BadRequest
                    })? {
                        content_length += chunk.len() as i64;
                        if let Err(err) = temp_file.write_all(&chunk).await {
                            tracing::error!("tempfile write error: {:#?}", err);
                            return Err(ChaptersError::InternalServerError);
                        }
                    }

                    temp_file.seek(SeekFrom::Start(0)).await.expect("seek file");

                    let s3_file_name = format!("{}_{}", file_name, Uuid::now_v7());

                    upload = upload
                        .path(s3_file_name)
                        .stream(ReaderStream::new(temp_file).map_err(|err| {
                            tracing::error!("tempfile stream error: {:#?}", err);
                            box_error(ChaptersError::InternalServerError)
                        }))
                        .content_type(
                            field
                                .content_type()
                                .ok_or_else(|| {
                                    tracing::error!("image field no content_type");
                                    ChaptersError::InternalServerError
                                })?
                                .to_string(),
                        );
                }
                _ => continue,
            }
        }
    }

    let chapter_page = chapter_page
        .build()
        .map_err(|_| ChaptersError::BadRequest)?;

    let upload = upload.build().map_err(|_| ChaptersError::BadRequest)?;

    let new_chapter_page = db
        .transaction(|transaction| {
            async move {
                // save chapter page to db
                let current_date = Utc::now().naive_utc();
                let chapter_page = entity::chapter_pages::Model {
                    id: Uuid::now_v7(),
                    author_id: user_claims.user.id,
                    comic_id: chapter_page.comic_id,
                    chapter_id: chapter_page.chapter_id,
                    number: chapter_page.number,
                    path: upload.path,
                    content_type: upload.content_type,
                    created_at: current_date,
                    updated_at: current_date,
                }
                .into_active_model()
                .insert(transaction)
                .await?;

                // upload image to s3
                if let Err(err) = storage
                    .put(
                        &chapter_page.path,
                        upload.stream,
                        content_length,
                        &chapter_page.content_type,
                    )
                    .await
                    .map_err(|err| {
                        tracing::debug!("s3 error: {:#?}", err);
                        ChaptersError::InternalServerError
                    })
                {
                    tracing::debug!("failed to upload to s3 rolling back");
                    return Err(err);
                }

                Ok(chapter_page)
            }
            .boxed()
        })
        // TODO: error handling
        .await?;

    let chapter_page = ChapterPageResponse {
        id: new_chapter_page.id,
        number: new_chapter_page.number,
        image: ImageResponse {
            path: new_chapter_page.path,
            content_type: new_chapter_page.content_type,
        },
    };

    Ok(Json(chapter_page))
}

/// Get chapters of a comic with pagination
#[utoipa::path(
    get,
    path = "/api/v1/chapters",
        params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "Get chapters of specified comic", body = [ChapterResponse]),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Chapters API"
)]
pub async fn get_chapters_cursor(
    State(db): State<DatabaseConnection>,
    Query(pagination): Query<PaginationParams>,
    Path(comic_id): Path<Uuid>,
) -> Result<Json<Vec<ChapterResponse>>, ChaptersError> {
    let chapters = entity::chapters::Entity::find()
        .filter(entity::chapters::Column::Id.gt(pagination.min_id))
        .filter(entity::chapters::Column::Id.lt(pagination.max_id))
        .filter(entity::chapters::Column::ComicId.eq(comic_id))
        .find_with_related(entity::chapter_pages::Entity)
        // TODO: determine good limit
        .limit(Some(10))
        .all(&db)
        .await?
        // TODO: error handling
        .into_iter()
        .map(|(chapter, pages)| ChapterResponse {
            id: chapter.id,
            number: chapter.number,
            description: chapter.description,
            created_at: chapter.created_at.to_string(),
            pages: pages
                .into_iter()
                .map(|page| ChapterPageResponse {
                    id: page.id,
                    number: page.number,
                    image: ImageResponse {
                        content_type: page.content_type,
                        path: page.path,
                    },
                })
                .collect(),
        })
        .collect();

    Ok(Json(chapters))
}

/// Get chapter of a comic
#[utoipa::path(
    get,
    path = "/api/v1/chapters/s/{chapter_id}",
    responses(
        (status = 200, description = "Get chapter", body = ChapterResponse),
        (status = StatusCode::NOT_FOUND, description = "Specified chapter not found", body = ErrorHandlingResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Chapters API"
)]
pub async fn get_chapter(
    State(db): State<DatabaseConnection>,
    Path(chapter_id): Path<Uuid>,
) -> Result<Json<ChapterResponse>, ChaptersError> {
    let chapter = entity::chapters::Entity::find_by_id(chapter_id)
        .find_with_related(entity::chapter_pages::Entity)
        .all(&db)
        .await?;

    let (chapter, chapter_pages) = chapter.into_iter().next().ok_or_else(|| {
        tracing::error!("No comic found");
        ChaptersError::ChapterNotFound
    })?;

    let chapter_pages = chapter_pages
        .into_iter()
        .map(|chapter_page| ChapterPageResponse {
            id: chapter.id,
            number: chapter.number,
            image: ImageResponse {
                content_type: chapter_page.content_type,
                path: chapter_page.path,
            },
        })
        .collect();

    let chapter = ChapterResponse {
        id: chapter.id,
        number: chapter.number,
        description: chapter.description,
        pages: chapter_pages,
        created_at: chapter.created_at.to_string(),
    };

    Ok(Json(chapter))
}

/// Update chapter
#[utoipa::path(
    put,
    path = "/api/v1/chapters/{chapter_id}",
    request_body(content = UpdateComic, content_type = "application/json"),
    responses(
        (status = 200, description = "Specified comic has been successfully updated", body = Uuid),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Chapters API"
)]
pub async fn update_chapter(
    State(db): State<DatabaseConnection>,
    Path(chapter_id): Path<Uuid>,
    Json(payload): Json<UpdateChapter>,
) -> Result<Json<Uuid>, ChaptersError> {
    let Some(chapter) = Chapter::find_by_id(chapter_id).one(&db).await? else {
        tracing::error!("Chapter not found: {}", chapter_id);
        return Err(ChaptersError::ChapterNotFound);
    };

    let mut chapter = chapter.into_active_model();

    if let Some(title) = payload.title {
        chapter.title = Set(Some(title));
    }

    if let Some(description) = payload.description {
        chapter.description = Set(Some(description));
    }

    if let Some(number) = payload.number {
        chapter.number = Set(number);
    }

    let chapter = chapter.update(&db).await?;
    // TODO: error handling

    Ok(Json(chapter.id))
}

/// Delete chapter
#[utoipa::path(
    delete,
    path = "/api/v1/chapters/{chapter_id}",
    responses(
        (status = 200, description = "Specified comic has been successfully deleted"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Chapters API"
)]
pub async fn delete_chapter(
    State(db): State<DatabaseConnection>,
    Path(chapter_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ChaptersError> {
    let res = Chapter::delete_by_id(chapter_id).exec(&db).await?;

    if res.rows_affected < 1 {
        tracing::error!("Chapter not found: {}", chapter_id);
        return Err(ChaptersError::ChapterNotFound);
    }

    Ok(Json(json!({
        "message": format!("deleted {} chapters", res.rows_affected)
    })))
}

/// Delete chapter page
#[utoipa::path(
    delete,
    path = "/api/v1/chapters/page/{chapter_page_id}",
    responses(
        (status = 200, description = "Specified chapter page has been successfully deleted"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Chapters API"
)]
pub async fn delete_chapter_page(
    State(db): State<DatabaseConnection>,
    Path(chapter_page_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ChaptersError> {
    let res = ChapterPage::delete_by_id(chapter_page_id).exec(&db).await?;

    if res.rows_affected < 1 {
        tracing::error!("Chapter page not found: {}", chapter_page_id);
        return Err(ChaptersError::ChapterNotFound);
    }

    Ok(Json(json!({
        "message": format!("deleted {} chapter pages", res.rows_affected)
    })))
}
