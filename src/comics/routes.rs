use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use itertools::multizip;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, LoaderTrait,
    ModelTrait, QueryFilter, QuerySelect, RelationTrait, Set, TransactionTrait,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::AuthExtractor,
    chapters::models::ChapterResponseBrief,
    comic_genres::{models::ComicGenre, ComicGenresError},
    entity::{
        self, chapters::Entity as Chapter, comic_genres::Entity as Genre, comics::Entity as Comic,
        comics_genres_mapping::Entity as GenreMapping, users::Entity as User,
    },
    users::models::UserResponseBrief,
    AppState, PaginationParams,
};

use super::{
    models::{ComicResponse, CreateComic, UpdateComic},
    ComicsError, ComicsParams,
};

pub fn comics_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_comic))
        .route("/:comic_id", get(get_comic))
        .route("/", get(get_comics_cursor))
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
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateComic>,
) -> Result<Json<ComicResponse>, ComicsError> {
    // save comic to db

    let transaction = db.begin().await?;

    let current_date = Utc::now().naive_utc();

    let comic = entity::comics::Model {
        id: Uuid::now_v7(),
        author_id: auth.current_user.id,
        title: payload.title,
        description: payload.description,
        created_at: current_date,
        updated_at: current_date,
    }
    .into_active_model()
    .insert(&transaction)
    .await
    .map_err(|e| {
        if let migration::DbErr::Query(sea_orm::RuntimeErr::SqlxError(sqlx::Error::Database(err))) =
            e
        {
            match err.constraint() {
                Some("comics_title_key") => {
                    tracing::error!("{}", err);
                    return ComicsError::Conflict(String::from(
                        "comic with same title already exists",
                    ));
                }
                _ => {
                    tracing::error!("{}", err);
                    return ComicsError::InternalServerError;
                }
            }
        }
        tracing::error!("{}", e);
        ComicsError::InternalServerError
    })?;

    let mut comic_response = ComicResponse {
        id: comic.id,
        author: auth.current_user,
        title: comic.title.to_string(),
        description: comic.description.to_string(),
        created_at: comic.created_at.to_string(),
        chapters: vec![],
        genres: vec![],
    };

    let Some(genres) = payload.categories else {
        return Ok(Json(comic_response));
    };

    let db_genre_mappings: Vec<entity::comics_genres_mapping::ActiveModel> = genres
        .iter()
        .map(|genre| {
            entity::comics_genres_mapping::Model {
                genre_id: *genre,
                comic_id: comic.id,
            }
            .into_active_model()
        })
        .collect();

    let _res = entity::comics_genres_mapping::Entity::insert_many(db_genre_mappings)
        .exec(&transaction)
        .await
        .map_err(|e| {
            if let sea_orm::error::DbErr::Exec(sea_orm::error::RuntimeErr::SqlxError(
                sqlx::Error::Database(error),
            )) = e
            {
                tracing::error!("db error: {}", error.message());
            }
            // TODO: handle conflict
            ComicsError::ComicGenresErrors(ComicGenresError::InvalidGenre)
        })?;

    transaction.commit().await?;

    let genres = comic.find_related(Genre).all(&db).await?;

    comic_response.genres = genres
        .into_iter()
        .map(|genre| ComicGenre {
            id: genre.id,
            name: genre.name,
        })
        .collect();

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
    State(db): State<DatabaseConnection>,
    Path(comic_id): Path<Uuid>,
) -> Result<Json<ComicResponse>, ComicsError> {
    let comic = Comic::find_by_id(comic_id)
        // .find_with_related(entity::chapters::Entity)
        .all(&db)
        .await?;

    let chapters = comic.load_many(Chapter, &db).await?;
    let genres = comic.load_many_to_many(Genre, GenreMapping, &db).await?;
    let user = comic.load_one(User, &db).await?;

    let (comic, genres, chapters, Some(user)) = multizip((comic, genres, chapters, user)).next().ok_or_else(|| {
        tracing::error!("No comic found");
        ComicsError::ComicNotFound
    })? else {
        tracing::error!("No comic author found");
        return Err(ComicsError::InternalServerError);
    };

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
    State(db): State<DatabaseConnection>,
    Query(params): Query<ComicsParams>,
) -> Result<Json<Vec<ComicResponse>>, ComicsError> {
    tracing::debug!("cursor: {:#?}", params);

    let mut comics_query = Comic::find()
        .filter(entity::comics::Column::Id.gt(params.min_id))
        .filter(entity::comics::Column::Id.lt(params.max_id))
        .join_rev(
            migration::JoinType::InnerJoin,
            entity::comics_genres_mapping::Entity::belongs_to(entity::comics::Entity)
                .from(entity::comics_genres_mapping::Column::ComicId)
                .to(entity::comics::Column::Id)
                .into(),
        )
        .join(
            migration::JoinType::InnerJoin,
            entity::comics_genres_mapping::Relation::ComicGenres.def(),
        )
        // TODO: determine good limit
        .limit(Some(10));

    if let Some(genre_filter) = params.genre {
        comics_query = comics_query.filter(entity::comic_genres::Column::Id.eq(genre_filter));
    }

    let comics = comics_query.all(&db).await?;

    let chapters = comics.load_many(entity::chapters::Entity, &db).await?;
    let genres = comics.load_many_to_many(Genre, GenreMapping, &db).await?;
    let users: Vec<entity::users::Model> = comics
        .load_one(User, &db)
        .await?
        .into_iter()
        .flatten()
        .collect();

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
                        .map(|genre| ComicGenre {
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
    State(db): State<DatabaseConnection>,
    Path(comic_id): Path<Uuid>,
    Json(payload): Json<UpdateComic>,
) -> Result<Json<Uuid>, ComicsError> {
    let Some(comic) = Comic::find_by_id(comic_id).one(&db).await? else {
        tracing::error!("Comic not found: {}", comic_id);
        return Err(ComicsError::ComicNotFound);
    };

    let mut comic = comic.into_active_model();

    if let Some(title) = payload.title {
        comic.title = Set(title);
    }

    if let Some(description) = payload.description {
        comic.description = Set(description);
    }

    let comic = comic.update(&db).await?;
    // TODO: error handling

    Ok(Json(comic.id))
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
    State(db): State<DatabaseConnection>,
    Path(comic_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ComicsError> {
    let res = Comic::delete_by_id(comic_id).exec(&db).await?;

    if res.rows_affected < 1 {
        tracing::error!("Comic not found: {}", comic_id);
        return Err(ComicsError::ComicNotFound);
    }

    Ok(Json(json!({
        "message": format!("deleted {} comics", res.rows_affected)
    })))
}
