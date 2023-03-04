use serde::Serialize;

#[derive(Serialize)]
pub struct PostResponse {
    pub content: String,
}
