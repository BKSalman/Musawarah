use crate::users::models::UserResponseBrief;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    comics::chapters::models::Chapter,
    schema::{chapter_comments, chapter_comments_mapping},
    users::models::User,
};

#[derive(Insertable, Queryable, Identifiable, Associations, Selectable, Debug)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Chapter))]
#[diesel(table_name = chapter_comments)]
pub struct ChapterComment {
    pub id: Uuid,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub chapter_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Insertable, Queryable, Identifiable, Associations, Selectable, Debug, Clone)]
#[diesel(belongs_to(ChapterComment, foreign_key = parent_comment_id))]
#[diesel(table_name = chapter_comments_mapping)]
#[diesel(primary_key(parent_comment_id, child_comment_id))]
pub struct ChapterCommentMapping {
    pub parent_comment_id: Uuid,
    pub child_comment_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, TS)]
#[ts(export)]
pub struct ChapterCommentResponse {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub content: String,
    pub user: UserResponseBrief,
    pub parent_comment: Option<Uuid>,
    pub child_comments_ids: Vec<Uuid>,
    pub child_comments: Vec<ChapterCommentResponse>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, TS)]
#[ts(export)]
pub struct CreateChapterComment {
    pub content: String,
    pub parent_comment_id: Option<Uuid>,
}
