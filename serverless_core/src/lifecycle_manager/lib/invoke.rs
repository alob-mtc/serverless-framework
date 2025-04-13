use crate::db::cache::FunctionCacheRepo;
use crate::db::function::FunctionDBRepo;
use crate::utils::utils::random_port;
use crate::lifecycle_manager::lib::error::{ServelessCoreError, ServelessCoreResult};
use redis::aio::MultiplexedConnection;
use runtime::core::runner::runner;
use sea_orm::DatabaseConnection;
use std::time::Duration;
use tracing::{error, info};

/// Checks if a function is registered in the database.
///
/// Returns `Ok(())` if the function exists; otherwise, returns an error
/// indicating that the function is not registered.
///
/// # Arguments
///
/// * `conn` - A reference to the database connection.
/// * `name` - The name of the function to check.
pub async fn check_function_status(
    conn: &DatabaseConnection,
    name: &str,
) -> ServelessCoreResult<()> {
    let function = FunctionDBRepo::find_function_by_name(conn, name).await;
    if function.is_none() {
        error!("Function '{}' not registered", name);
        return Err(ServelessCoreError::FunctionNotRegistered(name.to_string()));
    }
    Ok(())
}

/// Starts a function service if it's not already running.
///
/// This function first checks if the function is already running by querying the
/// cache repository. If a running instance is found, it returns the cached address.
/// Otherwise, it generates a random port, starts the function container using the
/// Docker runner, caches the new function's address, and returns it.
///
/// # Arguments
///
/// * `cache_conn` - A mutable reference to the Redis multiplexed connection.
/// * `name` - The name of the function to start.
///
/// # Returns
///
/// A `Result` containing the function's address (e.g., "localhost:PORT") on success,
/// or an error if the function fails to start.
pub async fn start_function(
    cache_conn: &mut MultiplexedConnection,
    name: &str,
) -> ServelessCoreResult<String> {
    // Check if the function is already running.
    if let Some(addr) = FunctionCacheRepo::get_function(cache_conn, name).await {
        info!("Function '{}' already running at: {}", name, addr);
        return Ok(addr);
    }

    // Generate a random port and prepare the service address.
    let port = random_port();
    let addr = format!("localhost:{}", port);
    let timeout = 10;

    // Attempt to run the function container with a timeout slightly longer than the cache TTL.
    match runner(
        name,
        &format!("{port}:8080"),
        Some(Duration::from_secs(timeout + 2)),
    )
    .await
    {
        Err(e) => {
            error!("Error starting function '{}': {:?}", name, e);
            Err(ServelessCoreError::FunctionFailedToStart(name.to_string()))
        }
        Ok(_) => {
            // Register the function in the cache.
            let _ = FunctionCacheRepo::add_function(cache_conn, name, &addr, timeout).await;
            info!("Function '{}' started at: {}", name, addr);
            Ok(addr)
        }
    }
}
