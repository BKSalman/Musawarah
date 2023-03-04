use axum::{
    routing::{get, post},
    Router,
};
use jwt_simple::prelude::HS256Key;
use once_cell::sync::Lazy;
use rmusawarah::{
    users::routes::{create_user, get_user, get_user_posts, login},
    AppState,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgrespw@localhost:55000")
        .await
        .expect("db connection");

    let app_state = AppState { db: pool };

    let user_router = Router::new()
        .route("/", post(create_user))
        .route("/login", post(login))
        .route("/:username", get(get_user_posts))
        .route("/", get(get_user));

    let app = Router::new()
        .route("/", get(root))
        .nest("/api/users", user_router)
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 6060));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("start server");
}

async fn root() -> &'static str {
    "Hello, World!"
}
