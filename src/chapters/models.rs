use std::fs;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::comics::models::ImageResponse;

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ChapterResponse {
    pub number: i32,
    pub description: Option<String>,
    pub pages: Vec<ChapterPageResponse>,
}

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ChapterPageResponse {
    pub number: i32,
    pub image: ImageResponse,
}

#[derive(ToSchema)]
#[allow(dead_code)]
// TODO: should this be CreateChapter or CreateChapterPage?
pub struct CreateChapter {
    title: String,
    description: String,
    #[schema(value_type = String, format = Binary)]
    image: fs::File,
}

#[derive(Builder, Deserialize, ToSchema)]
#[builder(pattern = "owned")]
pub struct ChapterData {
    pub title: String,
    pub description: String,
}

impl ChapterData {
    #[must_use]
    pub fn builder() -> ChapterDataBuilder {
        ChapterDataBuilder::default()
    }
}
