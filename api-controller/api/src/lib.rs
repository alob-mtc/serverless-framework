mod handlers;
use axum::{
    extract::FromRef,
    response::IntoResponse,
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

#[derive(Clone, FromRef)]
struct AppState {
    db_conn: DatabaseConnection,
    cache_conn: MultiplexedConnection,
}

pub async fn start_server() {
    dotenvy::dotenv().unwrap();
    tracing_subscriber::fmt::init();

    // connect to redis
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL is not set in .env file");
    let client = redis::Client::open(redis_url).unwrap();
    let cache_conn = client
        .get_multiplexed_async_connection()
        .await
        .expect("Cache connection failed");

    // connect to db
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db_conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    Migrator::up(&db_conn, None).await.unwrap();

    let app_state = AppState {
        db_conn,
        cache_conn,
    };

    let app = Router::new()
        .route("/upload", post(upload_function))
        .route("/service/:key", any(call_function))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("server listening on ... {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
