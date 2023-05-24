use crate::{comic_genres::models::ComicGenre, schema::comic_genres, AppState};
use futures_util::TryStreamExt;

use super::{models::Genre, ComicGenresError};
use axum::{extract::State, routing::get, Json, Router};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection, RunQueryDsl};

pub fn comic_genres_router() -> Router<AppState> {
    Router::new().route("/", get(get_comic_genres))
}

#[utoipa::path(
    get,
    path = "/api/v1/comic-genres",
    responses(
        (status = StatusCode::OK, body = [ComicGenre]),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Comic Categories API"
)]
#[axum::debug_handler]
pub async fn get_comic_genres(
    State(pool): State<Pool<AsyncPgConnection>>,
) -> Result<Json<Vec<ComicGenre>>, ComicGenresError> {
    let mut db = pool.get().await?;

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
