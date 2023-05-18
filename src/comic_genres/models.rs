use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, TS, PartialEq)]
pub struct ComicGenre {
    pub id: i32,
    pub name: String,
}
