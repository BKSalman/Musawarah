use axum::{extract::FromRef, response::IntoResponse};
use jwt_simple::prelude::HS256Key;
use once_cell::sync::Lazy;
use s3::interface::Storage;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    IntoParams, Modify, OpenApi, ToSchema,
};
use uuid::Uuid;

pub mod chapters;
pub mod comics;
pub mod entity;
pub mod middlewares;
pub mod s3;
pub mod users;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub storage: Storage,
}

pub static JWT_KEY: Lazy<HS256Key> = Lazy::new(|| HS256Key::generate());

#[derive(OpenApi)]
#[openapi(
    paths(
        users::routes::create_user,
        users::routes::login,
        users::routes::get_user_comics,
        users::routes::get_user,
        comics::routes::create_comic,
        comics::routes::update_comic,
        comics::routes::delete_comic,
        comics::routes::get_comic,
        comics::routes::get_comics_cursor,
        chapters::routes::create_chapter,
        chapters::routes::create_chapter_page,
        chapters::routes::get_chapters_cursor,
        chapters::routes::get_chapter,
        chapters::routes::delete_chapter,
        chapters::routes::delete_chapter_page,
        chapters::routes::update_chapter,
        // chapters::routes::update_chapter_page,
    ),
    components(
        schemas(comics::models::CreateComic),
        schemas(comics::models::UpdateComic),
        schemas(comics::models::ComicResponse),
        schemas(comics::models::ComicResponseBrief),
        schemas(comics::models::ImageResponse),
        schemas(chapters::models::CreateChapter),
        schemas(chapters::models::UpdateChapter),
        schemas(chapters::models::CreateChapterPage),
        schemas(chapters::models::ChapterResponse),
        schemas(chapters::models::ChapterPageResponse),
        schemas(users::models::UserResponse),
        schemas(users::models::UserClaims),
        schemas(users::models::CreateUser),
        schemas(users::models::CreateUserReponse),
        schemas(users::models::UserLogin),
        schemas(users::models::UserToken),
        schemas(ErrorHandlingResponse),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Users API"),
        (name = "Comics API"),
        (name = "Chapters API"),
    )
)]

pub struct ApiDoc;

#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationParams {
    #[serde(default = "Uuid::nil")]
    min_id: Uuid,
    #[serde(default = "Uuid::max")]
    max_id: Uuid,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ErrorHandlingResponse {
    pub errors: Vec<String>,
}

impl IntoResponse for ErrorHandlingResponse {
    fn into_response(self) -> axum::response::Response {
        serde_json::to_string(&self)
            .expect("ErrorHandlingResponse as json")
            .into_response()
    }
}

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            let bearer = HttpBuilder::new()
                .scheme(HttpAuthScheme::Bearer)
                .bearer_format("Bearer")
                .build();
            components.add_security_scheme("jwt", SecurityScheme::Http(bearer))
        }
    }
}

// TODO: add this

// #[derive(thiserror::Error, Debug)]
// pub enum CommonErrors {
//     #[error("internal server error")]
//     InternalServerError,
// }

// impl IntoResponse for CommonErrors {
//     fn into_response(self) -> axum::response::Response {
//         let (status, error_message) = match self {
//             CommonErrors::InternalServerError => (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 ErrorHandlingResponse {
//                     errors: vec![self.to_string()],
//                 },
//             ),
//         };

//         let body = Json(error_message);

//         (status, body).into_response()
//     }
// }
