use axum::{extract::FromRef, http::StatusCode, response::IntoResponse, Json};
use jwt_simple::prelude::HS256Key;
use once_cell::sync::Lazy;
use s3::interface::Storage;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, HttpBuilder, SecurityScheme},
    IntoParams, Modify, OpenApi, ToSchema,
};
use uuid::Uuid;

pub mod middlewares;
pub mod posts;
pub mod s3;
pub mod users;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: PgPool,
    pub storage: Storage,
}

pub static JWT_KEY: Lazy<HS256Key> = Lazy::new(|| HS256Key::generate());

#[derive(OpenApi)]
#[openapi(
    paths(
        users::routes::create_user,
        users::routes::login,
        users::routes::get_user_posts,
        users::routes::get_user,
        posts::routes::create_post,
        posts::routes::get_post,
        posts::routes::get_posts_cursor,
    ),
    components(
        schemas(posts::models::CreatePost),
        schemas(posts::models::PostData),
        schemas(posts::models::PostResponse),
        schemas(posts::models::ImageResponse),
        schemas(users::models::UserResponse),
        schemas(users::models::UserClaims),
        schemas(users::models::CreateUser),
        schemas(users::models::UserLogin),
        schemas(users::models::UserToken),
        schemas(ErrorHandlingResponse),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Users API"),
        (name = "Posts API")
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

#[derive(Serialize, Deserialize, ToSchema)]
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
