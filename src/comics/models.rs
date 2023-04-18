use std::fs;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::chapters::models::ChapterResponse;
use crate::users::models::UserResponse;

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ComicResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub author: UserResponse,
    pub chapters: Vec<ChapterResponse>,
}

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ImageResponse {
    pub content_type: String,
    pub path: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateComic {
    pub title: String,
    pub description: String,
}
