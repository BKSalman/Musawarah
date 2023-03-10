use axum::{
    routing::{get, post},
    Router,
};
use rmusawarah::{
    posts::routes::{create_post, get_post, get_posts},
    s3::helpers::setup_storage,
    users::routes::{create_user, get_user, get_user_posts, login},
    AppState,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgrespw@localhost:55000")
        .await
        .expect("db connection");

    let app_state = AppState {
        db: pool,
        storage: setup_storage().expect("storage"),
    };

    let user_router = Router::new()
        .route("/", get(get_user))
        .route("/", post(create_user))
        .route("/login", post(login))
        .route("/:username", get(get_user_posts));

    let posts_router = Router::new()
        .route("/", post(create_post))
        .route("/:post_id", get(get_post))
        .route("/cursor/:cursor", get(get_posts));

    let app = Router::new()
        .route("/", get(root))
        .nest("/api/users", user_router)
        .nest("/api/posts", posts_router)
        .layer(TraceLayer::new_for_http())
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
