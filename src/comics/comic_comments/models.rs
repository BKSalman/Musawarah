use crate::{
    comics::models::Comic,
    schema::{comic_comments, comic_comments_mapping},
    users::models::{User, UserResponseBrief},
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Insertable, Queryable, Identifiable, Associations, Selectable, Debug)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Comic))]
#[diesel(table_name = comic_comments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ComicComment {
    pub id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub comic_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Insertable, Queryable, Identifiable, Associations, Selectable, Debug, Clone)]
#[diesel(belongs_to(ComicComment, foreign_key = parent_comment_id))]
#[diesel(table_name = comic_comments_mapping)]
#[diesel(primary_key(parent_comment_id, child_comment_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ComicCommentMapping {
    pub parent_comment_id: Uuid,
    pub child_comment_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, TS)]
pub struct CreateComicComment {
    pub content: String,
    pub parent_comment_id: Option<Uuid>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, TS)]
#[ts(export)]
pub struct ComicCommentResponse {
    pub id: Uuid,
    pub comic_id: Uuid,
    pub content: String,
    pub user: UserResponseBrief,
    pub parent_comment: Option<Uuid>,
    pub child_comments_ids: Vec<Uuid>,
    pub child_comments: Vec<ComicCommentResponse>,
}
