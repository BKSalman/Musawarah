use diesel::Queryable;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Queryable, Debug, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ImageResponse {
    pub content_type: String,
    pub path: String,
}
