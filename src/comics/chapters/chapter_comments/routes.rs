use chrono::Utc;
use diesel::GroupedBy;
use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use diesel::BelongingToDsl;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use itertools::multizip;
use uuid::Uuid;

use crate::{
    auth::AuthExtractor,
    comics::chapters::{chapter_comments::models::CreateChapterComment, models::Chapter},
    schema::{chapter_comments, chapter_comments_mapping, comic_chapters, users},
    users::models::{User, UserResponseBrief, UserRole},
    AppState, InnerAppState,
};

use super::{
    models::{ChapterComment, ChapterCommentMapping, ChapterCommentResponse},
    ChapterCommentsError,
};

pub fn chapter_comments_router() -> Router<AppState> {
    Router::new()
        .route("/:chapter_id/comments", get(get_comments))
        .route("/:chapter_id/comments", post(create_comment))
        .route("/comments/:comment_id", delete(delete_comment))
}

#[utoipa::path(
    get,
    path = "/api/v1/comics/chapters/:chapter_id/comments",
    responses (
        (status = 200, body = ChapterCommentResponse),
        (status = StatusCode::BAD_REQUEST, description = "Invalid Chapter ID", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapter Comments API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_comments(
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
    Path(chapter_id): Path<Uuid>,
    State(state): State<Arc<InnerAppState>>,
) -> Result<Json<Vec<ChapterCommentResponse>>, ChapterCommentsError> {
    let mut db = state.pool.get().await?;

    let chapter = comic_chapters::table
        .filter(comic_chapters::id.eq(chapter_id))
        .get_result::<Chapter>(&mut db)
        .await?;

    let (comments, users): (Vec<ChapterComment>, Vec<User>) =
        ChapterComment::belonging_to(&chapter)
            .inner_join(users::table)
            .load::<(ChapterComment, User)>(&mut db)
            .await?
            .into_iter()
            .unzip();

    let comment_mappings = ChapterCommentMapping::belonging_to(&comments)
        .load::<ChapterCommentMapping>(&mut db)
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
        |(comment, user, comment_parent_mapping, comment_children_mapping)| {
            ChapterCommentResponse {
                id: comment.id,
                chapter_id: chapter.id,
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
            }
        },
    )
    .collect();

    Ok(Json(comments))
}

#[utoipa::path(
    post,
    path = "/api/v1/comics/chapters/:chapter_id/comments",
    request_body(content = CreateChapterComment, content_type = "application/json"),
    responses (
        (status = 200, description = "Comment successfully created", body = ChapterCommentResponse),
        (status = StatusCode::BAD_REQUEST, description = "Invalid Chapter ID", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapter Comments API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn create_comment(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(chapter_id): Path<Uuid>,
    Json(payload): Json<CreateChapterComment>,
) -> Result<Json<ChapterCommentResponse>, ChapterCommentsError> {
    let mut db = state.pool.get().await?;

    let comment = db
        .transaction::<_, ChapterCommentsError, _>(|transaction| {
            async move {
                let comment = ChapterComment {
                    id: Uuid::now_v7(),
                    content: payload.content,
                    created_at: Utc::now(),
                    updated_at: None,
                    chapter_id,
                    user_id: auth.current_user.id,
                };

                let comment = diesel::insert_into(chapter_comments::table)
                    .values(&comment)
                    .get_result::<ChapterComment>(transaction)
                    .await?;

                let commnet_response = ChapterCommentResponse {
                    id: comment.id,
                    chapter_id,
                    content: comment.content,
                    user: auth.current_user,
                    parent_comment: payload.parent_comment_id,
                    child_comments_ids: vec![],
                    child_comments: vec![],
                };

                if let Some(parent_comment_id) = payload.parent_comment_id {
                    diesel::insert_into(chapter_comments_mapping::table)
                        .values((
                            chapter_comments_mapping::parent_comment_id.eq(parent_comment_id),
                            chapter_comments_mapping::child_comment_id.eq(comment.id),
                        ))
                        .execute(transaction)
                        .await?;
                }

                Ok(commnet_response)
            }
            .scope_boxed()
        })
        .await?;

    Ok(Json(comment))
}

#[utoipa::path(
    delete,
    path = "/api/v1/comics/chapters/comments/:comment_id",
    responses(
        (status = 200, description = "Specified comment has been successfully deleted. returned deleted comment's ID", body = Uuid),
        (status = StatusCode::BAD_REQUEST, description = "Invalid Comment ID", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong", body = ErrorResponse),
    ),
    tag = "Chapter Comments API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn delete_comment(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(state): State<Arc<InnerAppState>>,
    Path(comment_id): Path<Uuid>,
) -> Result<Json<Uuid>, ChapterCommentsError> {
    let mut db = state.pool.get().await?;

    let comment = diesel::delete(
        chapter_comments::table
            .filter(chapter_comments::id.eq(comment_id))
            .filter(chapter_comments::user_id.eq(auth.current_user.id)),
    )
    .get_result::<ChapterComment>(&mut db)
    .await?;

    Ok(Json(comment.id))
}
