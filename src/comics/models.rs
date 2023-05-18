use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    chapters::models::ChapterResponseBrief, comic_genres::models::ComicGenre,
    users::models::UserResponseBrief,
};

#[derive(Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ComicResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub author: UserResponseBrief,
    pub chapters: Vec<ChapterResponseBrief>,
    pub genres: Vec<ComicGenre>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateComic {
    pub title: String,
    pub description: String,
    pub categories: Option<Vec<i32>>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateComic {
    pub title: Option<String>,
    pub description: Option<String>,
}
