use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use garde::Validate;
use itertools::multizip;
use uuid::Uuid;

use crate::{
    auth::AuthExtractor,
    comics::chapters::models::Chapter,
    comics::comic_genres::models::{ComicGenre, Genre, GenreMapping},
    comics::{models::NewComicRating, SortingOrder},
    schema::{comic_genres, comic_genres_mapping, comic_ratings, comics, users},
    users::models::{User, UserResponseBrief, UserRole},
    utils::average_rating,
    AppState, InnerAppState,
};

use super::{
    chapters::{models::ChapterPage, routes::chapters_router},
    comic_comments::routes::comic_comments_router,
    comic_genres::routes::comic_genres_router,
    models::{Comic, ComicRating, ComicResponse, CreateComic, UpdateComic},
    utils::slugify,
    ComicsError, ComicsParams,
};

pub fn comics_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_comic))
        .route("/", get(get_comics))
        .route("/:comic_id", put(update_comic))
        .route("/:comic_id", delete(delete_comic))
        .route("/:comic_id", get(get_comic))
        .route("/by_slug/:slug/:username", get(get_comic_by_slug))
        .route("/:comic_id/rate", post(rate_comic))
        .nest("/", comic_genres_router())
        .nest("/", comic_comments_router())
        .nest("/", chapters_router())
}

/// Create Comic
#[utoipa::path(
    post,
    path = "/api/v1/comics",
    request_body(content = CreateComic, content_type = "application/json"),
    responses(
        (status = 200, description = "Caller authorized. returned requested comic", body = ComicResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    security(
        ("auth" = [])
    ),
    tag = "Comics API"
)]
pub async fn create_comic(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Json(payload): Json<CreateComic>,
) -> Result<Json<ComicResponse>, ComicsError> {
    // save comic to db
    let mut db = state.pool.get().await?;

    let comic_response = db
        .transaction::<_, ComicsError, _>(|transaction| {
            async move {
                let comic = Comic {
                    id: Uuid::now_v7(),
                    slug: slugify(&payload.title),
                    user_id: auth.current_user.id,
                    title: payload.title,
                    description: payload.description,
                    is_visible: payload.is_visible,
                    published_at: None,
                    poster_path: None,
                    poster_content_type: None,
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
                    slug: comic.slug.to_string(),
                    description: comic.description.clone(),
                    rating: 0.0,
                    created_at: comic.created_at.to_string(),
                    chapters: vec![],
                    genres: vec![],
                };

                if let Some(genres) = payload.genres {
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
    path = "/api/v1/comics/:comic_id",
    responses(
        (status = 200, description = "Caller authorized. returned requested comic", body = ComicResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    security(
        ("auth" = [])
    ),
    tag = "Comics API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_comic(
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(comic_id): Path<Uuid>,
) -> Result<Json<ComicResponse>, ComicsError> {
    let mut db = state.pool.get().await?;

    let (comic, user) = comics::table
        .filter(comics::id.eq(comic_id))
        .inner_join(users::table)
        .select((Comic::as_select(), User::as_select()))
        .first::<(Comic, User)>(&mut db)
        .await?;

    let chapters = Chapter::belonging_to(&comic)
        .load::<Chapter>(&mut db)
        .await?;

    let chapter_pages = ChapterPage::belonging_to(&chapters)
        .select(ChapterPage::as_select())
        .load::<ChapterPage>(&mut db)
        .await?;

    let chapters_and_pages = chapter_pages
        .grouped_by(&chapters)
        .into_iter()
        .zip(chapters)
        .map(|(p, c)| (c, p))
        .collect::<Vec<(Chapter, Vec<ChapterPage>)>>();

    let genres = GenreMapping::belonging_to(&comic)
        .inner_join(comic_genres::table)
        .select(Genre::as_select())
        .load(&mut db)
        .await?;

    let comic_ratings = ComicRating::belonging_to(&comic)
        .select(ComicRating::as_select())
        .load(&mut db)
        .await?;

    let comic = ComicResponse {
        id: comic.id,
        author: UserResponseBrief {
            id: user.id,
            displayname: user.displayname,
            username: user.username,
            email: user.email,
            role: user.role,
        },
        title: comic.title,
        slug: comic.slug,
        description: comic.description,
        rating: average_rating(comic_ratings),
        created_at: comic.created_at.to_string(),
        chapters: chapters_and_pages
            .into_iter()
            .map(|(chapter, pages)| chapter.into_response_brief(pages))
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

/// Get comic by slug and username
#[utoipa::path(
    get,
    path = "/api/v1/comics/by_slug/:slug/:username",
    responses(
        (status = 200, description = "Caller authorized. returned requested comic", body = ComicResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Caller unauthorized", body = ErrorResponse ),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    security(
        ("auth" = [])
    ),
    tag = "Comics API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_comic_by_slug(
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path((slug, username)): Path<(String, String)>,
) -> Result<Json<ComicResponse>, ComicsError> {
    let mut db = state.pool.get().await?;

    let (comic, user) = comics::table
        .filter(comics::slug.eq(slug))
        .inner_join(users::table)
        .filter(users::username.eq(username))
        .select((Comic::as_select(), User::as_select()))
        .first::<(Comic, User)>(&mut db)
        .await?;

    let chapters = Chapter::belonging_to(&comic)
        .load::<Chapter>(&mut db)
        .await?;

    let chapter_pages = ChapterPage::belonging_to(&chapters)
        .select(ChapterPage::as_select())
        .load::<ChapterPage>(&mut db)
        .await?;

    let chapters_and_pages = chapter_pages
        .grouped_by(&chapters)
        .into_iter()
        .zip(chapters)
        .map(|(p, c)| (c, p))
        .collect::<Vec<(Chapter, Vec<ChapterPage>)>>();

    let genres = GenreMapping::belonging_to(&comic)
        .inner_join(comic_genres::table)
        .select(Genre::as_select())
        .load(&mut db)
        .await?;

    let comic_ratings = ComicRating::belonging_to(&comic)
        .select(ComicRating::as_select())
        .load(&mut db)
        .await?;

    let comic = ComicResponse {
        id: comic.id,
        author: UserResponseBrief {
            id: user.id,
            displayname: user.displayname,
            username: user.username,
            email: user.email,
            role: user.role,
        },
        title: comic.title,
        slug: comic.slug,
        description: comic.description,
        rating: average_rating(comic_ratings),
        created_at: comic.created_at.to_string(),
        chapters: chapters_and_pages
            .into_iter()
            .map(|(chapter, pages)| chapter.into_response_brief(pages))
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

/// Get comics with pagination and genre filtering
#[utoipa::path(
    get,
    path = "/api/v1/comics",
    params(
        ComicsParams,
    ),
    responses(
        (status = 200, body = [ComicResponse]),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Comics API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_comics(
    State(state): State<Arc<InnerAppState>>,
    Query(params): Query<ComicsParams>,
) -> Result<Json<Vec<ComicResponse>>, ComicsError> {
    tracing::debug!("cursor: {:#?}", params);
    let mut db = state.pool.get().await?;

    let mut comics_query = comics::table
        .inner_join(users::table)
        .left_join(comic_ratings::table)
        // PERF:: find a way to do the left join only when there's a genre_filter
        .left_join(comic_genres_mapping::table.inner_join(comic_genres::table))
        .filter(comics::id.gt(params.min_id))
        .filter(comics::id.lt(params.max_id))
        .into_boxed();

    if let Some(genre_filter) = params.genre {
        comics_query = comics_query.filter(comic_genres::id.eq(genre_filter));
    }

    if let Some(sorting_order) = params.sorting {
        match sorting_order {
            SortingOrder::Descending => {
                comics_query = comics_query.order(comic_ratings::rating.desc())
            }
            SortingOrder::Ascending => {
                comics_query = comics_query.order(comic_ratings::rating.asc())
            }
        }
    }

    let (comics, users): (Vec<Comic>, Vec<User>) = comics_query
        .distinct()
        .limit(10)
        .select((Comic::as_select(), User::as_select()))
        .load::<(Comic, User)>(&mut db)
        .await?
        .into_iter()
        .unzip();

    let chapters = Chapter::belonging_to(&comics)
        .load::<Chapter>(&mut db)
        .await?;

    let chapter_pages = ChapterPage::belonging_to(&chapters)
        .select(ChapterPage::as_select())
        .load::<ChapterPage>(&mut db)
        .await?;

    let chapters_and_pages = chapter_pages
        .grouped_by(&chapters)
        .into_iter()
        .zip(chapters)
        .map(|(p, c)| (c, p))
        .collect::<Vec<(Chapter, Vec<ChapterPage>)>>()
        .grouped_by(&comics);

    let genres: Vec<(GenreMapping, Genre)> = GenreMapping::belonging_to(&comics)
        .inner_join(comic_genres::table)
        .select((GenreMapping::as_select(), Genre::as_select()))
        .load::<(GenreMapping, Genre)>(&mut db)
        .await?;

    let genres = genres.grouped_by(&comics);

    let comics_ratings: Vec<ComicRating> = ComicRating::belonging_to(&comics)
        .select(ComicRating::as_select())
        .load::<ComicRating>(&mut db)
        .await?;

    let comics_ratings: Vec<Vec<ComicRating>> = comics_ratings.grouped_by(&comics);

    let comics: Result<Vec<ComicResponse>, ComicsError> =
        multizip((users, comics, genres, chapters_and_pages, comics_ratings))
            .map(|(user, comic, genres, chapter_and_pages, comic_ratings)| {
                Ok(ComicResponse {
                    id: comic.id,
                    title: comic.title,
                    slug: comic.slug,
                    description: comic.description,
                    created_at: comic.created_at.to_string(),
                    rating: average_rating(comic_ratings),
                    author: UserResponseBrief {
                        id: user.id,
                        displayname: user.displayname,
                        username: user.username,
                        email: user.email,
                        role: user.role,
                    },
                    chapters: chapter_and_pages
                        .into_iter()
                        .map(|(chapter, pages)| chapter.into_response_brief(pages))
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
    path = "/api/v1/comics/:comic_id",
    request_body(content = UpdateComic, content_type = "application/json"),
    responses(
        (status = 200, description = "Specified comic has been successfully updated", body = Uuid),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Comics API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn update_comic(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(comic_id): Path<Uuid>,
    Json(payload): Json<UpdateComic>,
) -> Result<Json<Uuid>, ComicsError> {
    let mut db = state.pool.get().await?;

    let updated_comic = diesel::update(
        comics::table
            .filter(comics::id.eq(comic_id))
            .filter(comics::user_id.eq(auth.current_user.id)),
    )
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
    path = "/api/v1/comics/:comic_id",
    responses(
        (status = 200, description = "Specified comic has been successfully deleted"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Comics API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn delete_comic(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(comic_id): Path<Uuid>,
) -> Result<Json<Uuid>, ComicsError> {
    let mut db = state.pool.get().await?;

    let deleted_comic = diesel::delete(
        comics::table
            .filter(comics::id.eq(comic_id))
            .filter(comics::user_id.eq(auth.current_user.id)),
    )
    .returning(Comic::as_returning())
    .get_result(&mut db)
    .await?;

    Ok(Json(deleted_comic.id))
}

/// Rate comic
#[utoipa::path(
    get,
    path = "/api/v1/comics/:comic_id/rate",
    request_body(content = NewComicRating, description = "Validation:\n- rating: 0-5", content_type = "application/json"),
    responses(),
    security(
        ("auth" = [])
    ),
    tag = "Comics API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn rate_comic(
    auth: AuthExtractor<{ UserRole::VerifiedUser as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(comic_id): Path<Uuid>,
    Json(payload): Json<NewComicRating>,
) -> Result<(), ComicsError> {
    payload.validate(&())?;

    let mut db = state.pool.get().await?;

    match diesel::update(
        comic_ratings::table
            .filter(comic_ratings::user_id.eq(auth.current_user.id))
            .filter(comic_ratings::comic_id.eq(comic_id)),
    )
    .set((
        comic_ratings::updated_at.eq(Some(Utc::now())),
        comic_ratings::rating.eq(payload.rating as f64),
    ))
    .get_result::<ComicRating>(&mut db)
    .await
    {
        Err(diesel::result::Error::NotFound) => {
            let comic_rating = ComicRating {
                id: Uuid::now_v7(),
                rating: payload.rating as f64,
                created_at: Utc::now(),
                updated_at: None,
                user_id: auth.current_user.id,
                comic_id,
            };

            diesel::insert_into(comic_ratings::table)
                .values(comic_rating)
                .execute(&mut db)
                .await?;

            Ok(())
        }
        Err(e) => Err(e.into()),
        Ok(_) => Ok(()),
    }
}
