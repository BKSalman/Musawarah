use std::fs;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::users::models::UserResponse;

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct PostResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub user: UserResponse,
    pub image: ImageResponse,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ImageResponse {
    pub content_type: String,
    pub path: String,
}

#[derive(ToSchema)]
#[allow(dead_code)]
pub struct CreatePost {
    title: String,
    content: String,
    #[schema(value_type = String, format = Binary)]
    image: fs::File,
}

#[derive(Builder, Deserialize, ToSchema)]
#[builder(pattern = "owned")]
pub struct PostData {
    pub title: String,
    pub content: String,
}

impl PostData {
    #[must_use]
    pub fn builder() -> PostDataBuilder {
        PostDataBuilder::default()
    }
}
