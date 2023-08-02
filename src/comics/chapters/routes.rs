use std::{io::SeekFrom, sync::Arc};

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use diesel::BelongingToDsl;
use diesel::GroupedBy;
use diesel::NullableExpressionMethods;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use futures::TryStreamExt;
use garde::Validate;
use itertools::multizip;
use serde::Deserialize;
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
    comics::chapters::{
        models::{ChapterPage, ChapterRating, NewChapterRating, UpdateChapterPage},
        ChaptersParams,
    },
    common::models::ImageResponse,
    s3::Upload,
    schema::{chapter_pages, chapter_ratings, comic_chapters, comics, users},
    users::models::UserRole,
    AppState, InnerAppState, SortingOrder,
};

use super::{
    chapter_comments::routes::chapter_comments_router,
    models::{
        Chapter, ChapterPageData, ChapterPageResponse, ChapterResponse, ChapterResponseBrief,
        CreateChapter, UpdateChapter,
    },
    utils::box_error,
    ChaptersError,
};

use uuid::Uuid;

const ALLOWED_MIME_TYPES: [&str; 3] = ["image/jpeg", "image/jpg", "image/png"];

pub const FILE_SIZE_LIMIT_MB: usize = 10;

const FILE_SIZE_LIMIT: usize = FILE_SIZE_LIMIT_MB * 1024 * 1024; // 10mb

pub fn chapters_router() -> Router<AppState> {
    Router::new()
        .layer(DefaultBodyLimit::disable())
        // TODO: image compression
        .layer(RequestBodyLimitLayer::new(FILE_SIZE_LIMIT))
        .route("/:comic_id/chapters", post(create_chapter))
        .route("/:comic_id/chapters", get(get_chapters))
        .route("/chapters/:chapter_id", delete(delete_chapter))
        .route("/chapters/:chapter_id", put(update_chapter))
        .route(
            "/chapters/by_slug/:username/:slug/:chapter_number/",
            get(get_chapter_by_slug),
        )
        .route("/chapters/:chapter_id/s", get(get_chapter))
        .route("/chapters/:chapter_id/rate", post(rate_chapter))
        .route(
            "/:comic_id/chapters/:chapter_id/pages",
            post(create_chapter_page),
        )
        .route("/chapters/pages/:chapter_page_id", put(update_chapter_page))
        .route(
            "/chapters/pages/:chapter_page_id",
            delete(delete_chapter_page),
        )
        .nest("/chapters/:chapter_id/comments", chapter_comments_router())
}

/// Create a chapter
#[utoipa::path(
    post,
    path = "/api/v1/comics/:comic_id/chapters",
    request_body(content = CreateChapter, content_type = "application/json"),
    responses(
        (status = 200, description = "Chapter successfully created", body = ChapterResponseBrief),
        (status = StatusCode::CONFLICT, description = "Chapter number conflicts with an already existing one", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapters API"
)]
pub async fn create_chapter(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(comic_id): Path<Uuid>,
    Json(payload): Json<CreateChapter>,
) -> Result<Json<ChapterResponseBrief>, ChaptersError> {
    let mut db = state.pool.get().await?;

    let chapter = Chapter {
        id: Uuid::now_v7(),
        user_id: auth.current_user.id,
        comic_id,
        number: payload.number,
        title: payload.title,
        description: payload.description,
        created_at: Utc::now(),
        updated_at: None,
        published_at: None,
        is_visible: false,
    };

    let chapter = diesel::insert_into(comic_chapters::table)
        .values(&chapter)
        .returning(Chapter::as_returning())
        .get_result::<Chapter>(&mut db)
        .await?;

    Ok(Json(chapter.into_response_brief(vec![])))
}

#[derive(Deserialize)]
pub struct ChapterPagePathParams {
    comic_id: Uuid,
    chapter_id: Uuid,
}

/// Create a chapter page
#[utoipa::path(
    post,
    path = "/api/v1/comics/:comic_id/chapters/:chapter_id/pages",
    request_body(content = CreateChapterPage, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Chapter page successfully created", body = ChapterPageResponse),
        (status = StatusCode::BAD_REQUEST, description = "Fields validation error", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapters API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn create_chapter_page(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(path_params): Path<ChapterPagePathParams>,
    mut fields: Multipart,
) -> Result<Json<ChapterPageResponse>, ChaptersError> {
    let mut db = state.pool.get().await?;

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
                    tracing::debug!("adding chapter page description");
                    chapter_page = chapter_page.description(field.text().await.ok());
                }
                "number" => {
                    tracing::debug!("adding chapter page number");
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
                "image" => {
                    tracing::debug!("adding chapter page image");
                    if !ALLOWED_MIME_TYPES
                        .contains(&field.content_type().ok_or(ChaptersError::BadRequest)?)
                    {
                        tracing::error!("wrong image type");
                        return Err(ChaptersError::BadRequest);
                    }

                    let file_name = field
                        .file_name()
                        .ok_or_else(|| {
                            tracing::error!("no file name");
                            ChaptersError::BadRequest
                        })?
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

                    let s3_file_name = format!("{}_{}", Uuid::now_v7(), file_name);

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

    tracing::debug!("{:#?}", chapter_page);

    let chapter_page = chapter_page.build().map_err(|e| {
        tracing::error!("failed to build chapter page: {}", e);

        ChaptersError::BadRequest
    })?;

    let upload = upload.build().map_err(|_| {
        tracing::error!("failed to build upload");

        ChaptersError::BadRequest
    })?;

    let new_chapter_page = db
        .transaction::<_, ChaptersError, _>(|transaction| {
            async move {
                // save chapter page to db
                let chapter_page = ChapterPage {
                    id: Uuid::now_v7(),
                    user_id: auth.current_user.id,
                    comic_id: path_params.comic_id,
                    chapter_id: path_params.chapter_id,
                    number: chapter_page.number,
                    description: chapter_page.description,
                    path: upload.path,
                    content_type: upload.content_type,
                    created_at: Utc::now(),
                    updated_at: None,
                };

                diesel::insert_into(chapter_pages::table)
                    .values(&chapter_page)
                    .execute(transaction)
                    .await?;

                // upload image to s3
                tracing::debug!("uploading chapter page image");
                if let Err(err) = state
                    .storage
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
        description: new_chapter_page.description,
        image: ImageResponse {
            path: new_chapter_page.path,
            content_type: new_chapter_page.content_type,
        },
    };

    Ok(Json(chapter_page))
}

/// Update chapter page
#[utoipa::path(
    put,
    path = "/api/v1/comics/chapters/pages/:chapter_page_id",
    request_body(content = UpdateChapter, content_type = "application/json"),
    responses(
        (status = 200, description = "Chapter page has successfully been updated", body = Uuid),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapters API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn update_chapter_page(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(chapter_page_id): Path<Uuid>,
    Json(payload): Json<UpdateChapterPage>,
) -> Result<Json<Uuid>, ChaptersError> {
    let mut db = state.pool.get().await?;

    let chapter_page = diesel::update(
        chapter_pages::table
            .filter(chapter_pages::id.eq(chapter_page_id))
            .filter(chapter_pages::user_id.eq(auth.current_user.id)),
    )
    .set(&payload)
    .returning(ChapterPage::as_returning())
    .get_result(&mut db)
    .await?;

    Ok(Json(chapter_page.id))
}

/// Get chapter of a comic
#[utoipa::path(
    get,
    path = "/api/v1/comics/chapters/:chapter_id/s/",
    responses(
        (status = 200, description = "Get chapter", body = ChapterResponse),
        (status = StatusCode::NOT_FOUND, description = "Specified chapter not found", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapters API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_chapter(
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(chapter_id): Path<Uuid>,
) -> Result<Json<ChapterResponse>, ChaptersError> {
    let mut db = state.pool.get().await?;

    let chapter = comic_chapters::table
        .find(chapter_id)
        .first::<Chapter>(&mut db)
        .await?;

    let chapter_pages = ChapterPage::belonging_to(&chapter)
        .order(chapter_pages::number.asc())
        .load::<ChapterPage>(&mut db)
        .await?;

    let chapter_ratings = ChapterRating::belonging_to(&chapter)
        .load::<ChapterRating>(&mut db)
        .await?;

    let chapter = chapter.into_response(chapter_pages, chapter_ratings);

    Ok(Json(chapter))
}

/// Get chapter of a comic by username, comic slug, and chapter number
#[utoipa::path(
    get,
    path = "/api/v1/comics/chapters/by_slug/:username/:slug/:chapter_number/",
    responses(
        (status = 200, description = "Get chapter", body = ChapterResponse),
        (status = StatusCode::NOT_FOUND, description = "Specified chapter not found", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapters API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_chapter_by_slug(
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path((username, slug, chapter_number)): Path<(String, String, i32)>,
) -> Result<Json<ChapterResponse>, ChaptersError> {
    let mut db = state.pool.get().await?;

    let comic_id = users::table
        .filter(users::username.eq(username))
        .inner_join(comics::table)
        .filter(comics::slug.eq(slug))
        .select(comics::id)
        .single_value();

    let chapter = comic_chapters::table
        .filter(comic_chapters::comic_id.nullable().eq(comic_id))
        .filter(comic_chapters::number.eq(chapter_number))
        .first::<Chapter>(&mut db)
        .await?;

    let chapter_pages = ChapterPage::belonging_to(&chapter)
        .order(chapter_pages::number.asc())
        .load::<ChapterPage>(&mut db)
        .await?;

    let chapter_ratings = ChapterRating::belonging_to(&chapter)
        .load::<ChapterRating>(&mut db)
        .await?;

    let chapter = chapter.into_response(chapter_pages, chapter_ratings);

    Ok(Json(chapter))
}

/// Update chapter
#[utoipa::path(
    put,
    path = "/api/v1/comics/chapters/:chapter_id",
    request_body(content = UpdateChapter, content_type = "application/json"),
    responses(
        (status = 200, description = "Chapter has successfully been updated", body = Uuid),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapters API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn update_chapter(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(chapter_id): Path<Uuid>,
    Json(payload): Json<UpdateChapter>,
) -> Result<Json<Uuid>, ChaptersError> {
    let mut db = state.pool.get().await?;

    let chapter = diesel::update(
        comic_chapters::table
            .filter(comic_chapters::id.eq(chapter_id))
            .filter(comic_chapters::user_id.eq(auth.current_user.id)),
    )
    .set(&payload)
    .returning(Chapter::as_returning())
    .get_result(&mut db)
    .await?;

    Ok(Json(chapter.id))
}

/// Delete chapter
#[utoipa::path(
    delete,
    path = "/api/v1/comics/chapters/:chapter_id",
    responses(
        (status = 200, description = "Specified chapter has been successfully deleted", body = Uuid),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapters API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn delete_chapter(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(chapter_id): Path<Uuid>,
) -> Result<Json<Uuid>, ChaptersError> {
    let mut db = state.pool.get().await?;

    let chapter = diesel::delete(
        comic_chapters::table
            .filter(comic_chapters::id.eq(chapter_id))
            .filter(comic_chapters::user_id.eq(auth.current_user.id)),
    )
    .get_result::<Chapter>(&mut db)
    .await?;

    Ok(Json(chapter.id))
}

/// Delete chapter page
#[utoipa::path(
    delete,
    path = "/api/v1/comics/chapters/page/:chapter_page_id",
    responses(
        (status = 200, description = "Specified chapter page has been successfully deleted"),
        (status = StatusCode::NOT_FOUND, description = "Specified chapter page was not found", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapters API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn delete_chapter_page(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(chapter_page_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ChaptersError> {
    let mut db = state.pool.get().await?;

    let res = diesel::delete(
        chapter_pages::table
            .filter(chapter_pages::id.eq(chapter_page_id))
            .filter(chapter_pages::user_id.eq(auth.current_user.id)),
    )
    .returning(ChapterPage::as_returning())
    .get_result(&mut db)
    .await?;

    Ok(Json(json!({
        "message": format!("deleted chapter page: {}", res.id)
    })))
}

/// Rate chapter
#[utoipa::path(
    get,
    path = "/api/v1/comics/:comic_id/chapters/:chapter_id/rate",
    request_body(content = NewChapterRating, description = "Validation:\n- rating: 0-10", content_type = "application/json"),
    responses(),
    security(
        ("auth" = [])
    ),
    tag = "Chapters API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn rate_chapter(
    auth: AuthExtractor<{ UserRole::VerifiedUser as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(chapter_id): Path<Uuid>,
    Json(payload): Json<NewChapterRating>,
) -> Result<(), ChaptersError> {
    payload.validate(&())?;

    let mut db = state.pool.get().await?;

    match diesel::update(
        chapter_ratings::table
            .filter(chapter_ratings::user_id.eq(auth.current_user.id))
            .filter(chapter_ratings::chapter_id.eq(chapter_id)),
    )
    .set((
        chapter_ratings::updated_at.eq(Some(Utc::now())),
        chapter_ratings::rating.eq(payload.rating as f64),
    ))
    .get_result::<ChapterRating>(&mut db)
    .await
    {
        Err(diesel::result::Error::NotFound) => {
            let chapter_rating = ChapterRating {
                id: Uuid::now_v7(),
                rating: payload.rating as f64,
                created_at: Utc::now(),
                updated_at: None,
                user_id: auth.current_user.id,
                chapter_id,
            };

            diesel::insert_into(chapter_ratings::table)
                .values(chapter_rating)
                .execute(&mut db)
                .await?;

            Ok(())
        }
        Err(e) => Err(e.into()),
        Ok(_) => Ok(()),
    }
}

/// Get chapters of a comic with pagination
#[utoipa::path(
    get,
    path = "/api/v1/comics/:comic_id/chapters",
        params(
        ChaptersParams
    ),
    responses(
        (status = 200, body = [ChapterResponse]),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapters API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_chapters(
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Query(params): Query<ChaptersParams>,
    Path(comic_id): Path<Uuid>,
) -> Result<Json<Vec<ChapterResponse>>, ChaptersError> {
    let mut db = state.pool.get().await?;

    let mut chapters_query = comic_chapters::table
        .left_join(chapter_ratings::table)
        .order(comic_chapters::number.asc())
        .filter(comic_chapters::comic_id.eq(comic_id))
        .filter(comic_chapters::id.gt(params.min_id))
        .filter(comic_chapters::id.lt(params.max_id))
        .into_boxed();

    if let Some(sorting_order) = params.sorting {
        match sorting_order {
            SortingOrder::Descending => {
                chapters_query = chapters_query.order(chapter_ratings::rating.desc())
            }
            SortingOrder::Ascending => {
                chapters_query = chapters_query.order(chapter_ratings::rating.asc())
            }
        }
    }

    let chapters = chapters_query
        .limit(10)
        .select(Chapter::as_select())
        .load::<Chapter>(&mut db)
        .await?;

    let chapter_pages = ChapterPage::belonging_to(&chapters)
        .select(ChapterPage::as_select())
        .load::<ChapterPage>(&mut db)
        .await?;

    let chapters_ratings = ChapterRating::belonging_to(&chapters)
        .select(ChapterRating::as_select())
        .load::<ChapterRating>(&mut db)
        .await?;

    let chapter_pages = chapter_pages.grouped_by(&chapters);
    let chapters_ratings = chapters_ratings.grouped_by(&chapters);

    let chapters = multizip((chapters, chapter_pages, chapters_ratings))
        .map(|(chapter, pages, chapter_ratings)| chapter.into_response(pages, chapter_ratings))
        .collect();

    Ok(Json(chapters))
}
