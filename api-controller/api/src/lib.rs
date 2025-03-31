mod handlers;
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
use std::env;
use std::net::SocketAddr;
use tracing::{error, info};

/// Application state shared across handlers.
#[derive(Clone, FromRef)]
struct AppState {
    /// Database connection for persisting data.
    db_conn: DatabaseConnection,
    /// Redis connection for caching.
    cache_conn: MultiplexedConnection,
}

/// Starts the server and sets up the necessary connections and routes.
///
/// This function performs the following:
/// - Loads environment variables from a `.env` file.
/// - Initializes structured logging.
/// - Connects to Redis and the database.
/// - Runs database migrations.
/// - Sets up the Axum router with defined routes.
/// - Binds the server to a socket address and starts serving requests.
pub async fn start_server() {
    // Load environment variables from .env file. Log an error if it fails.
    dotenvy::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();

    // Connect to Redis.
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL is not set in .env file");
    let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    let cache_conn = client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // Connect to the database.
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db_conn = Database::connect(db_url)
        .await
        .expect("Failed to connect to the database");

    // Run database migrations.
    Migrator::up(&db_conn, None)
        .await
        .expect("Database migration failed");

    let app_state = AppState {
        db_conn,
        cache_conn,
    };

    let app = Router::new()
        .route("/upload", post(upload_function))
        .route("/service/:key", any(call_function))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
