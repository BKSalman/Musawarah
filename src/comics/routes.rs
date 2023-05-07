use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use chrono::Utc;
use itertools::multizip;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, LoaderTrait,
    QueryFilter, QuerySelect, Set,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    chapters::models::ChapterResponseBrief,
    entity::{self, chapters::Entity as Chapter, comics::Entity as Comic, users::Entity as User},
    users::models::UserResponseBrief,
    PaginationParams,
};

use super::{
    models::{ComicResponseBrief, CreateComic, UpdateComic},
    ComicsError,
};

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
#[axum::debug_handler]
pub async fn create_comic(
    Extension(user): Extension<entity::users::Model>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateComic>,
) -> Result<Json<ComicResponseBrief>, ComicsError> {
    // save comic to db
    //     .map_err(|e| match e {
    //         sqlx::Error::Database(err) => match err.constraint() {
    //             Some("comics_title_key") => {
    //                 tracing::error!("{}", err);
    //                 ComicsError::Conflict(String::from("comic with same title already exists"))
    //             }
    //             _ => {
    //                 tracing::error!("{}", err);
    //                 ComicsError::InternalServerError
    //             }
    //         },
    //         _ => {
    //             tracing::error!("{}", e);
    //             ComicsError::InternalServerError
    //         }
    //     })?;

    let current_date = Utc::now().naive_utc();

    let comic = entity::comics::Model {
        id: Uuid::now_v7(),
        author_id: user.id,
        title: payload.title,
        description: payload.description,
        created_at: current_date,
        updated_at: current_date,
    }
    .into_active_model()
    .insert(&db)
    .await?;

    let comic = ComicResponseBrief {
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
        chapters: vec![],
    };

    Ok(Json(comic))
}

// #[derive(Deserialize, IntoParams)]
// pub struct ComicParams {
//     comic_id: Uuid,
// }

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
) -> Result<Json<ComicResponseBrief>, ComicsError> {
    let comic = Comic::find_by_id(comic_id)
        // .find_with_related(entity::chapters::Entity)
        .all(&db)
        .await?;

    let chapters = comic.load_many(Chapter, &db).await?;
    let user = comic.load_one(User, &db).await?;

    let (comic, chapters, Some(user)) = multizip((comic, chapters, user)).next().ok_or_else(|| {
        tracing::error!("No comic found");
        ComicsError::ComicNotFound
    })? else {
        tracing::error!("No comic author found");
        return Err(ComicsError::InternalServerError);
    };

    let comic = ComicResponseBrief {
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
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<ComicResponseBrief>>, ComicsError> {
    tracing::debug!("cursor: {:#?}", pagination);

    let comics = Comic::find()
        .filter(entity::comics::Column::Id.gt(pagination.min_id))
        .filter(entity::comics::Column::Id.lt(pagination.max_id))
        // TODO: determine good limit
        .limit(Some(10))
        .all(&db)
        .await?;

    let chapters = comics.load_many(entity::chapters::Entity, &db).await?;
    let users = comics.load_one(User, &db).await?;

    let comics: Result<Vec<ComicResponseBrief>, ComicsError> = multizip((users, comics, chapters))
        .filter(|(user, _comic, _chapters)| user.is_some())
        .map(|(user, comic, chapters)| {
            let user = user.ok_or(ComicsError::InternalServerError)?;
            Ok(ComicResponseBrief {
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
