use axum::{
    routing::{get, post},
    Extension, Router,
};
use rmusawarah::users::routes::{create_user, get_user};
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

    let user_router = Router::new()
        .route("/", post(create_user))
        .route("/:username", get(get_user))
        .layer(Extension(pool));

    let app = Router::new()
        .route("/", get(root))
        .nest("/api/users", user_router);

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
