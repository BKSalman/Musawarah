use axum::extract::FromRef;
use jwt_simple::prelude::HS256Key;
use once_cell::sync::Lazy;
use s3::interface::Storage;
use sqlx::PgPool;

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
