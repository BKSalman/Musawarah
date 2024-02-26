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

/// Get an image
#[utoipa::path(
    get,
    path = "/api/v1/images/:image_path",
    responses(
        (status = 200, description = "Image found", content_type = "application/octet-stream"),
        (status = StatusCode::BAD_REQUEST, description = "Image not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Something went wrong"),
    ),
    tag = "Images API"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_image(
    State(state): State<Arc<InnerAppState>>,
    // TODO: check if authorized to view the image (paid for the chapter that contains this image/page)
    _auth: AuthExtractor<{ UserRole::User as u32 }>,
    headers: HeaderMap,
    Path(image_path): Path<String>,
) -> Result<bytes::Bytes, ImagesError> {
    let referer = headers.get("referer").ok_or(ImagesError::BadRequest)?;

    if referer != &state.s3_referer {
        tracing::debug!("referer: {referer:?}");
        return Err(ImagesError::BadRequest);
    }

    let bytes = state.storage.get_bytes(&image_path).await?;

    Ok(bytes)
}
