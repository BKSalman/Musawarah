use std::io::SeekFrom;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use diesel::BelongingToDsl;
use diesel::GroupedBy;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::deadpool::Pool, scoped_futures::ScopedFutureExt, AsyncConnection,
    AsyncPgConnection, RunQueryDsl,
};
use futures::TryStreamExt;
use serde_json::json;
use tempfile::tempfile;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};
use tokio_util::io::ReaderStream;
use tower_http::limit::RequestBodyLimitLayer;

use crate::{
    auth::AuthExtractor,
    chapters::models::ChapterPage,
    common::models::ImageResponse,
    s3::{interface::Storage, Upload},
    schema::{chapter_pages, comic_chapters},
    AppState, PaginationParams,
};

use super::{
    models::{
        Chapter, ChapterPageData, ChapterPageResponse, ChapterResponse, ChapterResponseBrief,
        CreateChapter, UpdateChapter,
    },
    utils::box_error,
    ChaptersError,
};

use uuid::Uuid;

const ALLOWED_MIME_TYPES: [&str; 3] = ["image/jpeg", "image/jpg", "image/png"];

pub fn chapters_router() -> Router<AppState> {
    Router::new()
        .layer(DefaultBodyLimit::disable())
        // TODO: image compression
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024 /* 10mb */))
        .route("/", post(create_chapter))
        .route("/page", post(create_chapter_page))
        .route("/:comic_id", get(get_chapters_cursor))
        .route("/s/:chapter_id", get(get_chapter))
}

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
    auth: AuthExtractor,
    State(pool): State<Pool<AsyncPgConnection>>,
    Json(payload): Json<CreateChapter>,
) -> Result<Json<ChapterResponseBrief>, ChaptersError> {
    let mut db = pool.get().await?;

    let chapter = Chapter {
        id: Uuid::now_v7(),
        user_id: auth.current_user.id,
        comic_id: payload.comic_id,
        number: payload.number,
        title: payload.title,
        description: payload.description,
        created_at: Utc::now(),
        updated_at: None,
    };
    // .into_active_model()
    // .insert(&db)
    // .await
    // .map_err(|e| {
    //     if let migration::DbErr::Query(sea_orm::RuntimeErr::SqlxError(sqlx::Error::Database(err))) =
    //         e
    //     {
    //         match err.constraint() {
    //             Some("chapters_title_key") => {
    //                 tracing::error!("{}", err);
    //                 return ChaptersError::Conflict(String::from(
    //                     "chapter with same title already exists",
    //                 ));
    //             }
    //             Some("chapters_comic_id_number_key") => {
    //                 tracing::error!("{}", err);
    //                 return ChaptersError::Conflict(String::from(
    //                     "chapter with same number already exists",
    //                 ));
    //             }
    //             _ => {
    //                 tracing::error!("{}", err);
    //                 return ChaptersError::InternalServerError;
    //             }
    //         }
    //     }
    //     tracing::error!("DB error: {}", e);
    //     ChaptersError::InternalServerError
    // })?;

    let res = diesel::insert_into(comic_chapters::table)
        .values(&chapter)
        .returning(Chapter::as_returning())
        .get_result::<Chapter>(&mut db)
        .await?;

    Ok(Json(ChapterResponseBrief {
        id: res.id,
        number: res.number,
        description: res.description,
    }))
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
#[axum::debug_handler(state = AppState)]
pub async fn create_chapter_page(
    auth: AuthExtractor,
    State(storage): State<Storage>,
    State(pool): State<Pool<AsyncPgConnection>>,
    mut fields: Multipart,
) -> Result<Json<ChapterPageResponse>, ChaptersError> {
    let mut db = pool.get().await?;

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
        .transaction::<_, ChaptersError, _>(|transaction| {
            async move {
                // save chapter page to db
                let chapter_page = ChapterPage {
                    id: Uuid::now_v7(),
                    user_id: auth.current_user.id,
                    comic_id: chapter_page.comic_id,
                    chapter_id: chapter_page.chapter_id,
                    number: chapter_page.number,
                    path: upload.path,
                    content_type: upload.content_type,
                    created_at: Utc::now(),
                    updated_at: None,
                };

                diesel::insert_into(chapter_pages::table)
                    .values(&chapter_page)
                    .execute(transaction);

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
            .scope_boxed()
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
#[axum::debug_handler]
pub async fn get_chapters_cursor(
    State(pool): State<Pool<AsyncPgConnection>>,
    Query(pagination): Query<PaginationParams>,
    Path(comic_id): Path<Uuid>,
) -> Result<Json<Vec<ChapterResponse>>, ChaptersError> {
    let mut db = pool.get().await?;

    let chapters = comic_chapters::table
        .filter(comic_chapters::comic_id.eq(comic_id))
        .filter(comic_chapters::id.gt(pagination.min_id))
        .filter(comic_chapters::id.lt(pagination.max_id))
        .limit(10)
        .select(Chapter::as_select())
        .load::<Chapter>(&mut db)
        .await?;

    let chapter_pages = ChapterPage::belonging_to(&chapters)
        .select(ChapterPage::as_select())
        .load::<ChapterPage>(&mut db)
        .await?;

    let chapter_pages = chapter_pages.grouped_by(&chapters);

    let chapters = chapters
        .into_iter()
        .zip(chapter_pages.into_iter())
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
#[axum::debug_handler]
pub async fn get_chapter(
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(chapter_id): Path<Uuid>,
) -> Result<Json<ChapterResponse>, ChaptersError> {
    let mut db = pool.get().await?;

    let chapter = comic_chapters::table
        .find(chapter_id)
        .first::<Chapter>(&mut db)
        .await?;

    let chapter_pages = ChapterPage::belonging_to(&chapter)
        .load::<ChapterPage>(&mut db)
        .await?;

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
#[axum::debug_handler]
pub async fn update_chapter(
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(chapter_id): Path<Uuid>,
    Json(payload): Json<UpdateChapter>,
) -> Result<(), ChaptersError> {
    let mut db = pool.get().await?;

    let _chapter = diesel::update(comic_chapters::table.find(chapter_id))
        .set(&payload)
        .returning(Chapter::as_returning())
        .get_result(&mut db)
        .await?;
    // TODO: error handling

    Ok(())
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
#[axum::debug_handler]
pub async fn delete_chapter(
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(chapter_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ChaptersError> {
    let mut db = pool.get().await?;

    let _res = diesel::delete(comic_chapters::table.find(chapter_id)).execute(&mut db);

    Ok(Json(json!({ "message": format!("deleted chapter") })))
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
#[axum::debug_handler]
pub async fn delete_chapter_page(
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(chapter_page_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ChaptersError> {
    let mut db = pool.get().await?;

    let res = diesel::delete(chapter_pages::table.find(chapter_page_id))
        .returning(ChapterPage::as_returning())
        .get_result(&mut db)
        .await?;

    Ok(Json(json!({
        "message": format!("deleted chapter page: {}", res.id)
    })))
}
