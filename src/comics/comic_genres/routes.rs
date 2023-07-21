use std::sync::Arc;

use crate::{
    auth::AuthExtractor, comics::comic_genres::models::ComicGenre, schema::comic_genres,
    users::models::UserRole, AppState, InnerAppState,
};
use chrono::Utc;
use diesel::ExpressionMethods;
use futures_util::TryStreamExt;

use super::{
    models::{ComicGenreInsert, CreateComicGenre, Genre, UpdateComicGenre},
    ComicGenresError,
};
use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

pub fn comic_genres_router() -> Router<AppState> {
    Router::new()
        .route("/genres", get(get_genres))
        .route("/genres", post(create_genre))
        .route("/genres/:genre_id", put(update_genre))
        .route("/genres/:genre_id", delete(delete_genre))
}

#[utoipa::path(
    get,
    path = "/api/v1/comics/genres",
    responses(
        (status = StatusCode::OK, body = [ComicGenre]),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Comic Genres API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_genres(
    // _auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
) -> Result<Json<Vec<ComicGenre>>, ComicGenresError> {
    let mut db = state.pool.get().await?;

    let genres = comic_genres::table
        .select(Genre::as_select())
        .load_stream::<Genre>(&mut db)
        .await?
        .try_fold(Vec::new(), |mut acc, item| {
            acc.push(ComicGenre {
                id: item.id,
                name: item.name,
            });
            futures::future::ready(Ok(acc))
        })
        .await?;

    Ok(Json(genres))
}

#[utoipa::path(
    post,
    path = "/api/v1/comics/genres",
    responses(
        (status = StatusCode::OK),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Comic Genres API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn create_genre(
    _auth: AuthExtractor<{ UserRole::Staff as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Json(payload): Json<CreateComicGenre>,
) -> Result<(), ComicGenresError> {
    let mut db = state.pool.get().await?;

    diesel::insert_into(comic_genres::table)
        .values(ComicGenreInsert {
            name: payload.name,
            created_at: Utc::now(),
        })
        .execute(&mut db)
        .await?;

    Ok(())
}

#[utoipa::path(
    put,
    path = "/api/v1/comics/genres/:genre_id",
    responses(
        (status = StatusCode::OK),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Comic Genres API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn update_genre(
    _auth: AuthExtractor<{ UserRole::Admin as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(genre_id): Path<i32>,
    Json(payload): Json<UpdateComicGenre>,
) -> Result<(), ComicGenresError> {
    let mut db = state.pool.get().await?;

    diesel::update(comic_genres::table.filter(comic_genres::id.eq(genre_id)))
        .set(payload)
        .execute(&mut db)
        .await?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "/api/v1/comics/genres/:genre_id",
    responses(
        (status = StatusCode::OK),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Comic Genres API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn delete_genre(
    _auth: AuthExtractor<{ UserRole::Admin as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(genre_id): Path<i32>,
) -> Result<(), ComicGenresError> {
    let mut db = state.pool.get().await?;

    diesel::delete(comic_genres::table.filter(comic_genres::id.eq(genre_id)))
        .execute(&mut db)
        .await?;

    Ok(())
}
