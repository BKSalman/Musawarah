use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    chapters::models::ChapterResponseBrief,
    comic_genres::models::ComicGenre,
    schema::comics,
    users::models::{User, UserResponseBrief},
};

#[derive(Insertable, Queryable, Selectable, Associations, Identifiable, Debug, Clone)]
#[diesel(belongs_to(User))]
#[diesel(table_name = comics)]
pub struct Comic {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ComicResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub author: UserResponseBrief,
    pub chapters: Vec<ChapterResponseBrief>,
    pub genres: Vec<ComicGenre>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateComic {
    pub title: String,
    pub description: Option<String>,
    pub categories: Option<Vec<i32>>,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[diesel(table_name = comics)]
pub struct UpdateComic {
    pub title: Option<String>,
    pub description: Option<String>,
}
