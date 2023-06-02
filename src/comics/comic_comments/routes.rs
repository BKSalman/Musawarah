use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use diesel::BelongingToDsl;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{
    pooled_connection::deadpool::Pool, scoped_futures::ScopedFutureExt, AsyncConnection,
    AsyncPgConnection, RunQueryDsl,
};
use futures_util::TryStreamExt;
use uuid::Uuid;

use crate::{
    auth::AuthExtractor,
    comics::comic_comments::models::{ComicComment, ComicCommentResponse, CreateComicComment},
    comics::models::Comic,
    schema::{comic_comments, comic_comments_mapping, comics},
    users::models::UserRole,
    AppState,
};

use super::{models::ComicCommentMapping, ComicCommentsError};

pub fn comic_comments_router() -> Router<AppState> {
    Router::new().route("/:comic_id/comments", post(create_comment))
    // .route("/:comic_id", get(get_comments))
}

// #[utoipa::path(
//     get,
//     path = "/api/v1/comic-comments/:comic_id",
//     tag = "Comic Comments API"
// )]
// #[axum::debug_handler(state = AppState)]
// pub async fn get_comments(
//     _auth: AuthExtractor<{ UserRole::User as u32 }>,
//     State(pool): State<Pool<AsyncPgConnection>>,
//     Path(comic_id): Path<Uuid>,
// ) -> Result<Json<Vec<ComicCommentResponse>>, ComicCommentsError> {
//     let mut db = pool.get().await?;

//     let comic = comics::table
//         .filter(comics::id.eq(comic_id))
//         .get_result::<Comic>(&mut db)
//         .await?;

//     let comments = ComicComment::belonging_to(&comic)
//         .left_join(comic_comments_mapping::table)
//         .load_stream::<(ComicComment, Option<ComicCommentMapping>)>(&mut db)
//         .await?
//         .try_fold(Vec::new(), |mut acc, (comment, comment_mapping)| {
//             acc.push(ComicCommentResponse {
//                 content: comment.content,
//                 user_id: comment.user_id,
//                 // TODO: add this
//                 parent_comment_id: None,
//             });
//             futures::future::ready(Ok(acc))
//         })
//         .await?;

//     Ok(Json(comments))
// }

#[utoipa::path(
    post,
    path = "/api/v1/comics/:comic_id/comments",
    tag = "Comic Comments API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn create_comment(
    auth: AuthExtractor<{ UserRole::User as u32 }>,
    State(pool): State<Pool<AsyncPgConnection>>,
    Json(payload): Json<CreateComicComment>,
) -> Result<(), ComicCommentsError> {
    let mut db = pool.get().await?;

    db.transaction::<_, ComicCommentsError, _>(|transaction| {
        async move {
            let comment = ComicComment {
                id: Uuid::now_v7(),
                content: payload.content,
                created_at: Utc::now(),
                updated_at: None,
                comic_id: payload.comic_id,
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
            }

            Ok(())
        }
        .scope_boxed()
    })
    .await
}
