use axum::{
    extract::DefaultBodyLimit,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use rmusawarah::{
    chapters::routes::create_chapter,
    comics::routes::{create_comic, get_comic, get_comics_cursor},
    s3::helpers::setup_storage,
    users::routes::{create_user, get_user, get_user_comics, login},
    ApiDoc, AppState,
};
use sqlx::postgres::PgPoolOptions;
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

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    if let Err(err) = dotenv() {
        tracing::error!("Could not load .env file: {}", err);
    }

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env variable");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
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
        .route("/:username", get(get_user_comics));

    let comics_router = Router::new()
        .route("/", post(create_comic))
        .route("/", get(get_comics_cursor))
        .route("/:comic_id", get(get_comic));

    let chapters_router = Router::new().route("/", post(create_chapter));
    // .layer(DefaultBodyLimit::disable())
    // TODO: image compression
    // .layer(RequestBodyLimitLayer::new(5 * 1024 * 1024 /* 5mb */))
    // .route("/", get(get_comics_cursor))
    // .route("/:comic_id/:chapter_num", get(get_chapter));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_origin(Any);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .nest("/api/v1/users", user_router)
        .nest("/api/v1/comics", comics_router)
        .nest("/api/v1/chapters", chapters_router)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
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
