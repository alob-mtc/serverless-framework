mod config;
mod handlers;

use crate::config::ConfigError;
use axum::{
    extract::FromRef,
    routing::{any, post},
    Router,
};
use futures_util::stream::StreamExt;
use handlers::{call_function, upload_function};
use migration::{Migrator, MigratorTrait};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use sea_orm::{Database, DatabaseConnection};
use std::net::SocketAddr;
use thiserror::Error;
use tracing::{error, info};

// Re-export the config module
pub use config::AppConfig;

/// Application state shared across handlers.
#[derive(Clone, FromRef)]
pub struct AppState {
    /// Database connection for persisting data.
    db_conn: DatabaseConnection,
    /// Redis connection for caching.
    cache_conn: MultiplexedConnection,
    /// Application configuration
    config: AppConfig,
}

/// Custom error type for server initialization.
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("Redis connection error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Database connection error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    #[error("Server error: {0}")]
    ServerError(#[from] std::io::Error),

    #[error("Environment loading error: {0}")]
    EnvError(#[from] dotenvy::Error),

    #[error("HTTP server error: {0}")]
    HttpError(#[from] hyper::Error),
}

/// Starts the server and sets up the necessary connections and routes.
///
/// This function performs the following:
/// - Loads environment variables from a `.env` file.
/// - Initializes structured logging.
/// - Loads application configuration
/// - Connects to Redis and the database.
/// - Runs database migrations.
/// - Sets up the Axum router with defined routes.
/// - Binds the server to a socket address and starts serving requests.
pub async fn start_server() -> Result<(), ServerError> {
    // Load environment variables from .env file
    dotenvy::dotenv()?;
    tracing_subscriber::fmt::init();

    // Load application configuration
    let config = AppConfig::load()?;

    // Connect to Redis.
    let client = redis::Client::open(config.server.redis_url.clone())?;
    let cache_conn = client.get_multiplexed_async_connection().await?;

    // Connect to the database.
    let db_conn = Database::connect(config.server.database_url.clone()).await?;

    // Run database migrations.
    Migrator::up(&db_conn, None).await?;

    let app_state = AppState {
        db_conn,
        cache_conn,
        config: config.clone(),
    };

    let app = Router::new()
        .route("/upload", post(upload_function))
        .route("/service/:key", any(call_function))
        .with_state(app_state);

    // Build socket address from configuration
    let addr = SocketAddr::new(
        config
            .server
            .host
            .parse()
            .unwrap_or_else(|_| "0.0.0.0".parse().unwrap()),
        config.server.port,
    );

    info!("Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
