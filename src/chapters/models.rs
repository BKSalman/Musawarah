use std::fs;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{common::models::ImageResponse, entity};

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ChapterResponseBrief {
    pub id: Uuid,
    pub number: i32,
    pub description: Option<String>,
}

impl From<entity::chapters::Model> for ChapterResponseBrief {
    fn from(value: entity::chapters::Model) -> Self {
        Self {
            id: value.id,
            number: value.number,
            description: value.description,
        }
    }
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct CreateChapter {
    pub comic_id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub number: i32,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct UpdateChapter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub number: Option<i32>,
}

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ChapterResponse {
    pub id: Uuid,
    pub number: i32,
    pub description: Option<String>,
    pub pages: Vec<ChapterPageResponse>,
    pub created_at: String,
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
