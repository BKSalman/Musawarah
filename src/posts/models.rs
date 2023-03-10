use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::users::models::UserReponse;

#[derive(Serialize)]
pub struct PostResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub user: UserReponse,
    pub image: ImageResponse,
}

#[derive(Serialize)]
pub struct ImageResponse {
    pub content_type: String,
    pub path: String,
}

#[derive(Builder, Deserialize)]
#[builder(pattern = "owned")]
pub struct CreatePost {
    pub title: String,
    pub content: String,
}

impl CreatePost {
    #[must_use]
    pub fn builder() -> CreatePostBuilder {
        CreatePostBuilder::default()
    }
}
