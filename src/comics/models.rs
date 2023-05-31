use chrono::DateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    chapters::models::ChapterResponseBrief,
    comic_genres::models::ComicGenre,
    schema::{comic_ratings, comics},
    users::models::{User, UserResponseBrief},
    Rating,
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
    pub rating: f64,
    pub created_at: String,
    pub author: UserResponseBrief,
    pub chapters: Vec<ChapterResponseBrief>,
    pub genres: Vec<ComicGenre>,
}

#[derive(Insertable, Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Comic))]
#[diesel(table_name = comic_ratings)]
pub struct ComicRating {
    pub id: Uuid,
    pub rating: f64,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
    pub user_id: Uuid,
    pub comic_id: Uuid,
}

impl Rating for ComicRating {
    fn rating(&self) -> f64 {
        self.rating
    }
}

#[derive(Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
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

#[derive(Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct NewComicRating {
    pub rating: f64,
}
