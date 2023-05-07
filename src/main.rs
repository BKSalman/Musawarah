use axum::{
    extract::DefaultBodyLimit,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    routing::{get, post},
    Router,
};
use axum_login::{axum_sessions::SessionLayer, AuthLayer, RequireAuthorizationLayer};
use dotenv::dotenv;
use musawarah::{
    auth::SeaORMUserStore,
    chapters::routes::{create_chapter, create_chapter_page, get_chapter, get_chapters_cursor},
    comics::routes::{create_comic, get_comic, get_comics_cursor},
    entity,
    s3::helpers::setup_storage,
    sessions::SeaORMSessionStore,
    users::routes::{create_user, get_user, get_user_comics, login, logout},
    ApiDoc, AppState,
};
use rand::RngCore;
use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

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

    let db = sea_orm::Database::connect(database_url)
        .await
        .expect("Database connection");

    let session_store = SeaORMSessionStore::new(&db);

    let mut secret = [0u8; 64];

    rand::thread_rng().fill_bytes(&mut secret);

    #[allow(unused_mut)]
    let mut session_layer = SessionLayer::new(session_store, &secret);

    #[cfg(not(debug_assertions))]
    {
        session_layer = session_layer.with_secure(true);
    }

    type NoRoleAuthLayer = AuthLayer<SeaORMUserStore, Uuid, entity::users::Model, ()>;
    type RequireAuth = RequireAuthorizationLayer<Uuid, entity::users::Model, ()>;

    let user_store = SeaORMUserStore::new(&db);
    let auth_layer = NoRoleAuthLayer::new(user_store, &secret);

    let app_state = AppState {
        db,
        storage: setup_storage().expect("storage"),
    };

    let user_router = Router::new()
        .route("/comics/:username", get(get_user_comics))
        .route("/logout", get(logout))
        // prevents non-authenticated users from accessing this route
        .route_layer(RequireAuth::login())
        .route("/:user_id", get(get_user))
        .route("/", post(create_user))
        .route("/login", post(login));

    let comics_router = Router::new()
        .route("/", post(create_comic))
        .route("/", get(get_comics_cursor))
        .route("/:comic_id", get(get_comic));

    let chapters_router = Router::new()
        .layer(DefaultBodyLimit::disable())
        // TODO: image compression
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024 /* 10mb */))
        .route("/", post(create_chapter))
        .route("/page", post(create_chapter_page))
        .route_layer(RequireAuth::login())
        .route("/:comic_id", get(get_chapters_cursor))
        .route("/s/:chapter_id", get(get_chapter));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        // FIXME: add proper allowed origins
        .allow_origin(Any);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .nest("/api/v1/users", user_router)
        .nest("/api/v1/comics", comics_router)
        .nest("/api/v1/chapters", chapters_router)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(auth_layer)
        .layer(session_layer)
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
