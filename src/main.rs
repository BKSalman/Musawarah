use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    middleware,
    routing::get,
    Router,
};
use diesel_async::pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager};
use diesel_migrations_async::{embed_migrations, EmbeddedMigrations};
use dotenv::dotenv;
use musawarah::{
    chapters::routes::chapters_router, comic_genres::routes::comic_genres_router,
    comics::routes::comics_router, migrations::run_migrations, s3::helpers::setup_storage,
    sessions::refresh_session, users::routes::users_router, ApiDoc, AppState, COOKIES_SECRET,
};
use rand::Rng;
use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
};
use tower_cookies::{CookieManagerLayer, Key};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "debug");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    if let Err(err) = dotenv() {
        tracing::error!("Could not load .env file: {}", err);
    }

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env variable");

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = Pool::builder(config).build().expect("db connection pool");

    let mut db = pool.get().await.expect("db connection");

    run_migrations(&mut db).await.expect("Run migrations");

    drop(db);

    let app_state = AppState {
        pool,
        storage: setup_storage().expect("storage"),
    };

    // TODO: add to config file
    let mut secret = [0u8; 64];
    rand::thread_rng().fill(&mut secret);

    COOKIES_SECRET.set(Key::from(&secret)).ok();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        // FIXME: add proper allowed origins
        .allow_origin(Any);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .nest("/api/v1/users", users_router())
        .nest("/api/v1/comics", comics_router())
        .nest("/api/v1/comic-genres", comic_genres_router())
        .nest("/api/v1/chapters", chapters_router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            refresh_session,
        ))
        .layer(CookieManagerLayer::new())
        .with_state(app_state);

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 6060));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("start server");
}

async fn root() -> &'static str {
    "xqcL"
}
