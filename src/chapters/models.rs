use std::fs;

use chrono::DateTime;
use derive_builder::Builder;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    comics::models::Comic,
    common::models::ImageResponse,
    schema::{chapter_pages, comic_chapters},
    users::models::User,
};

#[derive(Insertable, Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Comic))]
#[diesel(table_name = comic_chapters)]
pub struct Chapter {
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub number: i32,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
    pub published_at: Option<DateTime<chrono::Utc>>,
    pub is_visible: bool,
    pub rating: Option<f64>,
    pub user_id: Uuid,
    pub comic_id: Uuid,
}

#[derive(Insertable, Queryable, Selectable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(Comic))]
#[diesel(belongs_to(Chapter))]
#[diesel(belongs_to(User))]
#[diesel(table_name = chapter_pages)]
pub struct ChapterPage {
    pub id: Uuid,
    pub number: i32,
    pub path: String,
    pub content_type: String,
    pub comic_id: Uuid,
    pub chapter_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct CreateChapter {
    pub comic_id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub number: i32,
}

#[derive(AsChangeset, Deserialize, ToSchema, Debug)]
#[diesel(table_name = comic_chapters)]
pub struct UpdateChapter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub number: Option<i32>,
}

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ChapterResponse {
    pub id: Uuid,
    pub title: Option<String>,
    pub number: i32,
    pub description: Option<String>,
    pub pages: Vec<ChapterPageResponse>,
    pub created_at: DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ChapterResponseBrief {
    pub id: Uuid,
    pub title: Option<String>,
    pub number: i32,
    pub description: Option<String>,
    pub created_at: DateTime<chrono::Utc>,
}

#[derive(ToSchema)]
#[allow(dead_code)]
pub struct CreateChapterPage {
    chapter_id: Uuid,
    comic_id: Uuid,
    number: u32,
    #[schema(value_type = String, format = Binary)]
    image: fs::File,
}

#[derive(Builder, Deserialize, ToSchema)]
#[builder(pattern = "owned")]
pub struct ChapterPageData {
    pub chapter_id: Uuid,
    pub comic_id: Uuid,
    pub number: i32,
    pub description: Option<String>,
}

impl ChapterPageData {
    #[must_use]
    pub fn builder() -> ChapterPageDataBuilder {
        ChapterPageDataBuilder::default()
    }
}

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ChapterPageResponse {
    pub id: Uuid,
    pub number: i32,
    pub image: ImageResponse,
}
