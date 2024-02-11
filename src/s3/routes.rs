use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    routing::get,
    Router,
};

use crate::{auth::AuthExtractor, users::models::UserRole, AppState, InnerAppState};

use super::ImagesError;

pub fn images_routes() -> Router<AppState> {
    Router::new().route("/:image_path", get(get_image))
}

pub async fn get_image(
    State(state): State<Arc<InnerAppState>>,
    // TODO: check if authorized
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
    headers: HeaderMap,
    Path(image_path): Path<String>,
) -> Result<Vec<u8>, ImagesError> {
    let referer = headers.get("referer").ok_or(ImagesError::BadRequest)?;

    // TODO: make this configurable and not hard-coded
    if referer != "http://localhost:5173/" {
        tracing::debug!("referer: {referer:?}");
        return Err(ImagesError::BadRequest);
    }

    let bytes = state.storage.get_bytes(&image_path).await.map_err(|e| {
        tracing::error!("failed to get image bytes: {e}");
        ImagesError::BadRequest
    })?;

    Ok(bytes.to_vec())
}
