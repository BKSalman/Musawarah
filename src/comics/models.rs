use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    chapters::models::ChapterResponseBrief,
    entity,
    users::models::{UserResponse, UserResponseBrief},
};

#[derive(Serialize, Deserialize, ToSchema, TS, sqlx::FromRow)]
#[ts(export)]
pub struct ComicResponseBrief {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub chapters: Vec<ChapterResponseBrief>,
    pub author: UserResponseBrief,
}

#[derive(Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ComicResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub chapters: Vec<ChapterResponseBrief>,
    pub author: UserResponse,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, TS, sqlx::FromRow, FromQueryResult)]
#[ts(export)]
pub struct ImageResponse {
    pub content_type: String,
    pub path: String,
}

impl From<entity::profile_images::Model> for ImageResponse {
    fn from(value: entity::profile_images::Model) -> Self {
        Self {
            path: value.path,
            content_type: value.content_type,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateComic {
    pub title: String,
    pub description: String,
}
