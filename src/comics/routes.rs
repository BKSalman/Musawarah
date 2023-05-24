use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{
    pooled_connection::deadpool::Pool, scoped_futures::ScopedFutureExt, AsyncConnection,
    AsyncPgConnection, RunQueryDsl,
};
use itertools::multizip;
use uuid::Uuid;

use crate::{
    auth::AuthExtractor,
    chapters::models::{Chapter, ChapterResponseBrief},
    comic_genres::models::{ComicGenre, Genre, GenreMapping},
    schema::{comic_genres, comic_genres_mapping, comics, users},
    users::models::{User, UserResponseBrief},
    AppState, PaginationParams,
};

use super::{
    models::{Comic, ComicResponse, CreateComic, UpdateComic},
    ComicsError, ComicsParams,
};

pub fn comics_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_comic))
        .route("/", get(get_comics_cursor))
        .route("/:comic_id", put(update_comic))
        .route("/:comic_id", delete(delete_comic))
        .route("/:comic_id", get(get_comic))
}

/// Create Comic
#[utoipa::path(
    post,
    path = "/api/v1/comics",
    request_body(content = CreateComic, content_type = "application/json"),
    responses(
        (status = 200, description = "Caller authorized. returned requested comic", body = ComicResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("auth" = [])
    ),
    tag = "Comics API"
)]
pub async fn create_comic(
    auth: AuthExtractor,
    State(pool): State<Pool<AsyncPgConnection>>,
    Json(payload): Json<CreateComic>,
) -> Result<Json<ComicResponse>, ComicsError> {
    // save comic to db
    let mut db = pool.get().await?;

    let comic_response = db
        .transaction::<_, ComicsError, _>(|transaction| {
            async move {
                let comic = Comic {
                    id: Uuid::now_v7(),
                    user_id: auth.current_user.id,
                    title: payload.title,
                    description: payload.description,
                    created_at: Utc::now(),
                    updated_at: None,
                };

                let comic = diesel::insert_into(comics::table)
                    .values(&comic)
                    .returning(Comic::as_returning())
                    .get_result(transaction)
                    .await?;

                let mut comic_response = ComicResponse {
                    id: comic.id,
                    author: auth.current_user,
                    title: comic.title.to_string(),
                    description: comic.description.clone(),
                    created_at: comic.created_at.to_string(),
                    chapters: vec![],
                    genres: vec![],
                };

                if let Some(genres) = payload.categories {
                    let db_genre_mappings: Vec<GenreMapping> = genres
                        .iter()
                        .map(|genre| GenreMapping {
                            comic_id: comic.id,
                            genre_id: *genre,
                        })
                        .collect();

                    diesel::insert_into(comic_genres_mapping::table)
                        .values(&db_genre_mappings)
                        .execute(transaction)
                        .await?;

                    let genres = GenreMapping::belonging_to(&comic)
                        .inner_join(comic_genres::table)
                        .select(Genre::as_select())
                        .load::<Genre>(transaction)
                        .await?;

                    comic_response.genres = genres
                        .into_iter()
                        .map(|genre| ComicGenre {
                            id: genre.id,
                            name: genre.name,
                        })
                        .collect();
                }
                Ok(comic_response)
            }
            .scope_boxed()
        })
        .await?;

    Ok(Json(comic_response))
}

/// Get comic by id
#[utoipa::path(
    get,
    path = "/api/v1/comics/{comic_id}",
    responses(
        (status = 200, description = "Caller authorized. returned requested comic", body = ComicResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorHandlingResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    security(
        ("auth" = [])
    ),
    tag = "Comics API"
)]
#[axum::debug_handler]
pub async fn get_comic(
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(comic_id): Path<Uuid>,
) -> Result<Json<ComicResponse>, ComicsError> {
    let mut db = pool.get().await?;

    let (comic, user) = comics::table
        .filter(comics::id.eq(comic_id))
        .inner_join(users::table)
        .select((Comic::as_select(), User::as_select()))
        .first::<(Comic, User)>(&mut db)
        .await?;

    let chapters = Chapter::belonging_to(&comic)
        .select(Chapter::as_select())
        .load(&mut db)
        .await?;

    let genres = GenreMapping::belonging_to(&comic)
        .inner_join(comic_genres::table)
        .select(Genre::as_select())
        .load(&mut db)
        .await?;

    let comic = ComicResponse {
        id: comic.id,
        author: UserResponseBrief {
            id: user.id,
            displayname: user.displayname,
            username: user.username,
            email: user.email,
        },
        title: comic.title,
        description: comic.description,
        created_at: comic.created_at.to_string(),
        chapters: chapters
            .into_iter()
            .map(|chapter| ChapterResponseBrief {
                id: chapter.id,
                number: chapter.number,
                description: chapter.description,
            })
            .collect(),
        genres: genres
            .into_iter()
            .map(|genre| ComicGenre {
                id: genre.id,
                name: genre.name,
            })
            .collect(),
    };

    Ok(Json(comic))
}

/// Get comics with pagination
#[utoipa::path(
    get,
    path = "/api/v1/comics",
    params(
        PaginationParams,
    ),
    responses(
        (status = 200, body = [ComicResponse]),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Comics API"
)]
#[axum::debug_handler]
pub async fn get_comics_cursor(
    State(pool): State<Pool<AsyncPgConnection>>,
    Query(params): Query<ComicsParams>,
) -> Result<Json<Vec<ComicResponse>>, ComicsError> {
    tracing::debug!("cursor: {:#?}", params);
    let mut db = pool.get().await?;

    let mut query = comics::table
        .inner_join(users::table)
        .left_join(comic_genres_mapping::table.inner_join(comic_genres::table))
        .limit(10)
        .filter(comics::id.gt(params.min_id))
        .filter(comics::id.lt(params.max_id))
        .into_boxed();

    if let Some(genre_filter) = params.genre {
        query = query.filter(comic_genres::id.eq(genre_filter));
    }

    let (comics, users): (Vec<Comic>, Vec<User>) = query
        .select((Comic::as_select(), User::as_select()))
        .load::<(Comic, User)>(&mut db)
        .await?
        .into_iter()
        .unzip();

    let chapters = Chapter::belonging_to(&comics)
        .load::<Chapter>(&mut db)
        .await?;

    let chapters = chapters.grouped_by(&comics);

    let genres: Vec<(GenreMapping, Genre)> = GenreMapping::belonging_to(&comics)
        .inner_join(comic_genres::table)
        .select((GenreMapping::as_select(), Genre::as_select()))
        .load::<(GenreMapping, Genre)>(&mut db)
        .await?;

    let genres = genres.grouped_by(&comics);

    let comics: Result<Vec<ComicResponse>, ComicsError> =
        multizip((users, comics, genres, chapters))
            .map(|(user, comic, genres, chapters)| {
                Ok(ComicResponse {
                    id: comic.id,
                    title: comic.title,
                    description: comic.description,
                    created_at: comic.created_at.to_string(),
                    author: UserResponseBrief {
                        id: user.id,
                        displayname: user.displayname,
                        username: user.username,
                        email: user.email,
                    },
                    chapters: chapters
                        .into_iter()
                        .map(|chapter| ChapterResponseBrief {
                            id: chapter.id,
                            number: chapter.number,
                            description: chapter.description,
                        })
                        .collect(),
                    genres: genres
                        .into_iter()
                        .map(|(_genre_mapping, genre)| ComicGenre {
                            id: genre.id,
                            name: genre.name,
                        })
                        .collect(),
                })
            })
            .collect();

    Ok(Json(comics?))
}

/// Update comic
#[utoipa::path(
    put,
    path = "/api/v1/comics/{comic_id}",
    request_body(content = UpdateComic, content_type = "application/json"),
    responses(
        (status = 200, description = "Specified comic has been successfully updated", body = Uuid),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Comics API"
)]
#[axum::debug_handler]
pub async fn update_comic(
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(comic_id): Path<Uuid>,
    Json(payload): Json<UpdateComic>,
) -> Result<Json<Uuid>, ComicsError> {
    let mut db = pool.get().await?;

    let updated_comic = diesel::update(comics::table.find(comic_id))
        .set(payload)
        .returning(Comic::as_returning())
        .get_result(&mut db)
        .await
        .map_err(|e| {
            if let diesel::result::Error::NotFound = e {
                return ComicsError::ComicNotFound;
            }
            e.into()
        })?;
    // TODO: error handling

    Ok(Json(updated_comic.id))
}

/// Delete comic
#[utoipa::path(
    delete,
    path = "/api/v1/comics/{comic_id}",
    responses(
        (status = 200, description = "Specified comic has been successfully deleted"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorHandlingResponse),
    ),
    tag = "Comics API"
)]
#[axum::debug_handler]
pub async fn delete_comic(
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(comic_id): Path<Uuid>,
) -> Result<Json<Uuid>, ComicsError> {
    let mut db = pool.get().await?;

    let deleted_comic = diesel::delete(comics::table.find(comic_id))
        .returning(Comic::as_returning())
        .get_result(&mut db)
        .await?;

    Ok(Json(deleted_comic.id))
}
