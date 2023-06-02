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
use dotenvy::dotenv;
use musawarah::{
    comics::routes::comics_router, migrations::run_migrations, s3::helpers::setup_storage,
    sessions::refresh_session, users::routes::users_router, ApiDoc, AppState, Config, ConfigError,
    COOKIES_SECRET,
};
use rand::Rng;
use std::{
    env, fs,
    net::{Ipv4Addr, SocketAddr},
};
use tower_cookies::{CookieManagerLayer, Key};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_appender::rolling;
use tracing_subscriber::{fmt::writer::MakeWriterExt, EnvFilter};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[tokio::main]
async fn main() {
    logging();

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

    let config = match Config::load_config() {
        Ok(config) => config,
        Err(err) => match &err {
            ConfigError::IoError(err) if err.kind() == std::io::ErrorKind::NotFound => {
                tracing::warn!("GENERATING CONFIG FILE WITH SECRET");

                let mut secret = [0u8; 64];
                rand::thread_rng().fill(&mut secret);

                let secret = String::from_utf8_lossy(&secret).to_string();

                let config = Config {
                    cookie_secret: secret,
                };

                let config_str = toml::to_string(&config).unwrap();

                fs::write("config.toml", config_str).unwrap();

                config
            }
            _ => {
                panic!("{:#?}", err);
            }
        },
    };

    COOKIES_SECRET
        .set(Key::from(config.cookie_secret.as_bytes()))
        .ok();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        // FIXME: add proper allowed origins
        .allow_origin([
            "http://locahost:6060".parse().unwrap(),
            "http://localhost:5173".parse().unwrap(),
        ])
        .allow_credentials(true);

    let v1_router = Router::new()
        .nest("/api/v1/users", users_router())
        .nest("/api/v1/comics", comics_router());

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .merge(v1_router)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            refresh_session,
        ))
        .layer(CookieManagerLayer::new())
        .with_state(app_state);

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 6060));

    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("start server");
}

fn logging() {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "debug");

    // Log all `tracing` events to files prefixed with `debug`. Since these
    // files will be written to very frequently, roll the log file every minute.
    let debug_file = rolling::minutely("./logs", "debug");
    // Log warnings and errors to a separate file. Since we expect these events
    // to occur less frequently, roll that file on a daily basis instead.
    let warn_file = rolling::daily("./logs", "warnings").with_max_level(tracing::Level::WARN);
    let all_files = debug_file.and(warn_file);

    tracing_subscriber::fmt()
        .with_writer(all_files)
        .with_writer(std::io::stderr)
        .compact()
        .with_line_number(true)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

async fn root() -> &'static str {
    "xqcL"
}
