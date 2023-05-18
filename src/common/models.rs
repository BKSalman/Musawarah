use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::entity;

#[derive(Debug, Serialize, Deserialize, ToSchema, TS, FromQueryResult)]
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
