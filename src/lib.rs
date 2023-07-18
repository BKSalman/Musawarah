use std::{fmt::Display, fs};

use axum::{extract::FromRef, response::IntoResponse};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};
use once_cell::sync::OnceCell;
use s3::interface::Storage;
use serde::{Deserialize, Serialize};
use tower_cookies::cookie::Key;
use ts_rs::TS;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi, ToSchema,
};

pub mod auth;
pub mod comics;
pub mod common;
pub mod migrations;
pub mod s3;
pub mod schema;
pub mod sessions;
pub mod users;
pub mod utils;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    TomlError(#[from] toml::de::Error),
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub cookie_secret: String,
    pub email_username: String,
    pub email_password: String,
    pub email_smtp_server: String,
}

impl Config {
    pub fn load_config() -> Result<Self, ConfigError> {
        tracing::info!("getting config file");
        let config_file = fs::read_to_string("config.toml")?;
        toml::from_str::<Config>(&config_file).map_err(Into::into)
    }
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: Pool<AsyncPgConnection>,
    pub storage: Storage,
}

pub static COOKIES_SECRET: OnceCell<Key> = OnceCell::new();
pub static EMAIL_USERNAME: OnceCell<String> = OnceCell::new();
pub static EMAIL_PASSWORD: OnceCell<String> = OnceCell::new();
pub static EMAIL_SMTP_SERVER: OnceCell<String> = OnceCell::new();

#[derive(OpenApi)]
#[openapi(
    paths(
        users::routes::create_user,
        users::routes::login,
        users::routes::logout,
        users::routes::get_user_comics,
        users::routes::get_user,
        users::routes::me,
        comics::routes::create_comic,
        comics::routes::update_comic,
        comics::routes::delete_comic,
        comics::routes::get_comic,
        comics::routes::get_comics,
        comics::routes::rate_comic,
        comics::chapters::routes::create_chapter,
        comics::chapters::routes::create_chapter_page,
        comics::chapters::routes::get_chapters,
        comics::chapters::routes::get_chapter,
        comics::chapters::routes::delete_chapter,
        comics::chapters::routes::delete_chapter_page,
        comics::chapters::routes::update_chapter,
        comics::chapters::routes::rate_chapter,
        // chapters::routes::update_chapter_page,
        comics::comic_genres::routes::get_genres,
        comics::comic_genres::routes::create_genre,
        comics::comic_genres::routes::update_genre,
        comics::comic_genres::routes::delete_genre,
        comics::comic_comments::routes::get_comments,
        comics::comic_comments::routes::create_comment,
        comics::comic_comments::routes::delete_comment,
    ),
    components(
        schemas(common::models::ImageResponse),
        schemas(comics::models::CreateComic),
        schemas(comics::models::UpdateComic),
        schemas(comics::models::ComicResponse),
        schemas(comics::models::NewComicRating),
        schemas(comics::comic_genres::models::ComicGenre),
        schemas(comics::chapters::models::CreateChapter),
        schemas(comics::chapters::models::UpdateChapter),
        schemas(comics::chapters::models::CreateChapterPage),
        schemas(comics::chapters::models::ChapterResponse),
        schemas(comics::chapters::models::ChapterResponseBrief),
        schemas(comics::chapters::models::ChapterPageResponse),
        schemas(comics::chapters::models::NewChapterRating),
        schemas(comics::comic_comments::models::ComicCommentResponse),
        schemas(users::models::UserRole),
        schemas(users::models::UserResponseBrief),
        schemas(users::models::UserResponse),
        schemas(users::models::UserClaims),
        schemas(users::models::CreateUser),
        schemas(users::models::UserLogin),
        schemas(users::models::UserToken),
        schemas(ErrorResponse),
        schemas(SortingOrder),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Users API"),
        (name = "Comics API"),
        (name = "Chapters API"),
        (name = "Comic Genres API"),
    )
)]
pub struct ApiDoc;

#[derive(Serialize, Deserialize, ToSchema, Debug, Default, TS)]
#[ts(export)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<Vec<String>>,
}

impl ErrorResponse {
    pub fn new(error_message: impl Display) -> Self {
        Self {
            error: error_message.to_string(),
            ..Default::default()
        }
    }

    pub fn with_details(error_message: impl Display, details: Vec<impl Display>) -> Self {
        Self {
            error: error_message.to_string(),
            details: Some(details.iter().map(|s| s.to_string()).collect()),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        serde_json::to_string(&self)
            .expect("ErrorResponse as json")
            .into_response()
    }
}

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            let http_auth_scheme = HttpBuilder::new().scheme(HttpAuthScheme::Basic).build();
            components.add_security_scheme("auth", SecurityScheme::Http(http_auth_scheme))
        }
    }
}

pub trait Rating {
    fn rating(&self) -> f64;
}

#[derive(Debug, Deserialize, ToSchema)]
pub enum SortingOrder {
    #[serde(rename = "desc")]
    Descending,
    #[serde(rename = "asc")]
    Ascending,
}
