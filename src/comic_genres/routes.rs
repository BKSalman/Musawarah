use axum::{extract::State, routing::get, Json, Router};
use futures::TryStreamExt;
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{entity::comic_genres::Entity as Genre, AppState};

use super::{models::ComicGenre, ComicGenresError};

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
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<ComicGenre>>, ComicGenresError> {
    let genres = Genre::find()
        .stream(&db)
        .await
        .map_err(|e| {
            tracing::error!("db error: {}", e);
            ComicGenresError::PlaceHolder
        })?
        .and_then(|genre| async move {
            Ok(ComicGenre {
                id: genre.id,
                name: genre.name,
            })
        })
        .try_collect::<Vec<ComicGenre>>()
        .await
        .map_err(|e| {
            tracing::error!("stream error: {}", e);
            ComicGenresError::PlaceHolder
        })?;

    Ok(Json(genres))
}
