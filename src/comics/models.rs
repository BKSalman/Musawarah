use chrono::DateTime;
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
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
    pub rating: Option<f64>,
    pub is_visible: bool,
    pub published_at: Option<DateTime<chrono::Utc>>,
    pub poster_path: Option<String>,
    pub poster_content_type: Option<String>,
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
    pub genres: Option<Vec<i32>>,
    pub is_visible: bool,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[diesel(table_name = comics)]
pub struct UpdateComic {
    pub title: Option<String>,
    pub description: Option<String>,
}
