use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use diesel::BelongingToDsl;
use diesel::GroupedBy;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use itertools::multizip;
use uuid::Uuid;

use crate::{
    auth::AuthExtractor,
    comics::comic_comments::models::{ComicComment, ComicCommentResponse, CreateComicComment},
    comics::models::Comic,
    schema::{comic_comments, comic_comments_mapping, comics, users},
    users::models::{User, UserResponseBrief, UserRole},
    AppState, InnerAppState,
};

use super::{models::ComicCommentMapping, ComicCommentsError};

pub fn comic_comments_router() -> Router<AppState> {
    Router::new()
        .route("/:comic_id/comments", post(create_comment))
        .route("/:comic_id/comments", get(get_comments))
        .route("/comments/:comment_id", delete(delete_comment))
}

#[utoipa::path(
    get,
    path = "/api/v1/comics/:comic_id/comments",
    responses(
        (status = 200, description = "Caller authorized, returned comics comments", body = [ComicCommentResponse]),
        (status = StatusCode::BAD_REQUEST, description = "Invalid Comic ID", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Comic Comments API",
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_comments(
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(comic_id): Path<Uuid>,
) -> Result<Json<Vec<ComicCommentResponse>>, ComicCommentsError> {
    let mut db = state.pool.get().await?;

    let comic = comics::table
        .filter(comics::id.eq(comic_id))
        .get_result::<Comic>(&mut db)
        .await?;

    let (comments, users): (Vec<ComicComment>, Vec<User>) = ComicComment::belonging_to(&comic)
        .inner_join(users::table)
        .load::<(ComicComment, User)>(&mut db)
        .await?
        .into_iter()
        .unzip();

    let comment_mappings = ComicCommentMapping::belonging_to(&comments)
        .load::<ComicCommentMapping>(&mut db)
        .await?;

    let comment_mappings_by_parents = comment_mappings.clone().grouped_by(&comments); // 1 parent -> children

    let id_indices: HashMap<_, _> = comments
        .iter()
        .enumerate()
        .map(|(i, u)| (u.id, i))
        .collect();

    let mut comment_mappings_by_children = comments.iter().map(|_| Vec::new()).collect::<Vec<_>>();

    for child in comment_mappings {
        comment_mappings_by_children[id_indices[&child.child_comment_id]].push(child);
    }

    let comments = multizip((
        comments,
        users,
        comment_mappings_by_parents,
        comment_mappings_by_children,
    ))
    .map(
        |(comment, user, comment_parent_mapping, comment_children_mapping)| ComicCommentResponse {
            id: comment.id,
            comic_id: comic.id,
            content: comment.content,
            user: UserResponseBrief {
                id: user.id,
                displayname: user.displayname,
                username: user.username,
                email: user.email,
                role: user.role,
            },
            parent_comment: comment_children_mapping.get(0).map(|m| m.parent_comment_id),
            child_comments_ids: comment_parent_mapping
                .iter()
                .map(|m| m.child_comment_id)
                .collect(),
            child_comments: vec![],
        },
    )
    .collect();

    Ok(Json(comments))
}

// #[utoipa::path(
//     get,
//     path = "/api/v1/comics/comments/:comment_id",
//     tag = "Comic Comments API"
// )]
// #[axum::debug_handler(state = AppState)]
// pub async fn get_comment(
//     _auth: AuthExtractor<{ UserRole::User as u32 }>,
//     State(pool): State<Pool<AsyncPgConnection>>,
//     Path(comment_id): Path<Uuid>,
// ) -> Result<Json<ComicCommentResponse>, ComicCommentsError> {
//     let mut db = pool.get().await?;

//     let comment = comic_comments::table
//         .filter(comic_comments::id.eq(comment_id))
//         .inner_join(users::table)
//         .left_join()
//         .first::<ComicComment>(&mut db)
//         .await?;

//     let comment = ComicCommentResponse {
//         id: comment.id,
//         content: comment.content,
//         user: UserResponseBrief {
//             id: user.id,
//             displayname: user.displayname,
//             username: user.username,
//             email: user.email,
//             role: user.role,
//         },
//         child_comments: comment_mapping,
//     };

//     Ok(Json(comment))
// }

#[utoipa::path(
    post,
    path = "/api/v1/comics/:comic_id/comments",
    request_body(content = CreateComicComment, content_type = "application/json"),
    responses (
        (status = 200, description = "Comment successfully created", body = ComicCommentResponse),
        (status = StatusCode::BAD_REQUEST, description = "Invalid Comic ID", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Comic Comments API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn create_comment(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(comic_id): Path<Uuid>,
    Json(payload): Json<CreateComicComment>,
) -> Result<Json<ComicCommentResponse>, ComicCommentsError> {
    let mut db = state.pool.get().await?;

    let comment = db
        .transaction::<_, ComicCommentsError, _>(|transaction| {
            async move {
                let comment = ComicComment {
                    id: Uuid::now_v7(),
                    content: payload.content,
                    created_at: Utc::now(),
                    updated_at: None,
                    comic_id,
                    user_id: auth.current_user.id,
                };

                let comment = diesel::insert_into(comic_comments::table)
                    .values(&comment)
                    .get_result::<ComicComment>(transaction)
                    .await?;

                if let Some(parent_comment_id) = payload.parent_comment_id {
                    diesel::insert_into(comic_comments_mapping::table)
                        .values((
                            comic_comments_mapping::parent_comment_id.eq(parent_comment_id),
                            comic_comments_mapping::child_comment_id.eq(comment.id),
                        ))
                        .execute(transaction)
                        .await?;

                    Ok(ComicCommentResponse {
                        id: comment.id,
                        comic_id,
                        content: comment.content,
                        user: auth.current_user,
                        parent_comment: Some(parent_comment_id),
                        child_comments_ids: vec![],
                        child_comments: vec![],
                    })
                } else {
                    Ok(ComicCommentResponse {
                        id: comment.id,
                        comic_id,
                        content: comment.content,
                        user: auth.current_user,
                        parent_comment: None,
                        child_comments_ids: vec![],
                        child_comments: vec![],
                    })
                }
            }
            .scope_boxed()
        })
        .await?;

    Ok(Json(comment))
}

#[utoipa::path(
    delete,
    path = "/api/v1/comics/comments/:comment_id",
    responses(
        (status = 200, description = "Specified comment has been successfully deleted. returned deleted comment's ID", body = Uuid),
        (status = StatusCode::BAD_REQUEST, description = "Invalid Comment ID", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Comic Comments API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn delete_comment(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(comment_id): Path<Uuid>,
) -> Result<Json<Uuid>, ComicCommentsError> {
    let mut db = state.pool.get().await?;

    let comment = diesel::delete(
        comic_comments::table
            .filter(comic_comments::id.eq(comment_id))
            .filter(comic_comments::user_id.eq(auth.current_user.id)),
    )
    .get_result::<ComicComment>(&mut db)
    .await?;

    Ok(Json(comment.id))
}
