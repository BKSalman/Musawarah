use crate::{
    comics::models::Comic,
    schema::{comic_comments, comic_comments_mapping},
    users::models::User,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Insertable, Queryable, Identifiable, Associations, Selectable, Debug)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Comic))]
#[diesel(table_name = comic_comments)]
pub struct ComicComment {
    pub id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub comic_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Insertable, Queryable, Identifiable, Associations, Selectable, Debug)]
#[diesel(belongs_to(ComicComment, foreign_key = parent_comment_id))]
#[diesel(table_name = comic_comments_mapping)]
#[diesel(primary_key(parent_comment_id, child_comment_id))]
pub struct ComicCommentMapping {
    pub parent_comment_id: Uuid,
    pub child_comment_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateComicComment {
    pub comic_id: Uuid,
    pub content: String,
    pub parent_comment_id: Option<Uuid>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ComicCommentResponse {
    pub content: String,
    pub user_id: Uuid,
    pub parent_comment_id: Option<Uuid>,
}
