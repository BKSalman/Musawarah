use axum::BoxError;
use derive_builder::Builder;
use serde::Deserialize;

pub mod helpers;
pub mod interface;
pub mod models;

pub type Result<T, E = BoxError> = std::result::Result<T, E>;

#[derive(Builder, Deserialize)]
#[builder(pattern = "owned")]
pub struct Upload<S> {
    pub path: String,
    pub content_type: String,
    pub stream: S,
}

impl<S> Upload<S> {
    #[must_use]
    pub fn builder() -> UploadBuilder<S> {
        UploadBuilder::default()
    }
}
