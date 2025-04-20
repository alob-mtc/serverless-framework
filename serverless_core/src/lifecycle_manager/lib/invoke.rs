use crate::db::cache::FunctionCacheRepo;
use crate::db::function::FunctionDBRepo;
use crate::lifecycle_manager::lib::error::{ServelessCoreError, ServelessCoreResult};
use crate::utils::utils::{generate_hash, random_port};
use md5;
use redis::aio::MultiplexedConnection;
use runtime::core::runner::runner;
use sea_orm::DatabaseConnection;
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

/// Checks if a function is registered in the database.
///
/// Returns `Ok(())` if the function exists; otherwise, returns an error
/// indicating that the function is not registered.
///
/// # Arguments
///
/// * `conn` - A reference to the database connection.
/// * `name` - The name of the function to check.
/// * `user_uuid` - The UUID of the user (namespace) to verify function ownership.
pub async fn check_function_status(
    conn: &DatabaseConnection,
    name: &str,
    user_uuid: Uuid,
) -> ServelessCoreResult<()> {
    let function = FunctionDBRepo::find_function_by_name(conn, name, user_uuid).await;
    if function.is_none() {
        error!("Function '{}' not found in namespace '{}'", name, user_uuid);
        return Err(ServelessCoreError::FunctionNotRegistered(format!(
            "Function '{}' not found in namespace '{}'",
            name, user_uuid
        )));
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
/// * `user_uuid` - The UUID of the user (namespace) who owns this function.
///
/// # Returns
///
/// A `Result` containing the function's address (e.g., "localhost:PORT") on success,
/// or an error if the function fails to start.
pub async fn start_function(
    cache_conn: &mut MultiplexedConnection,
    name: &str,
    user_uuid: Uuid,
) -> ServelessCoreResult<String> {
    // Generate a shorter hash of the UUID for better container names
    let uuid_short = generate_hash(user_uuid);

    // Create a unique function name based on function name and user's UUID hash
    let function_key = format!("{name}-{uuid_short}");

    // Check if the function is already running.
    if let Some(addr) = FunctionCacheRepo::get_function(cache_conn, &function_key).await {
        info!(
            "Function '{}' for user '{}' already running at: {}",
            name, user_uuid, addr
        );
        return Ok(addr);
    }

    // Generate a random port and prepare the service address.
    let port = random_port();
    let addr = format!("localhost:{}", port);
    let timeout = 10;

    // Attempt to run the function container with a timeout slightly longer than the cache TTL.
    match runner(
        &function_key,
        &format!("{port}:8080"),
        Some(Duration::from_secs(timeout + 2)),
    )
    .await
    {
        Err(e) => {
            error!(
                "Error starting function '{}' for user '{}': {:?}",
                name, user_uuid, e
            );
            Err(ServelessCoreError::FunctionFailedToStart(name.to_string()))
        }
        Ok(_) => {
            // Register the function in the cache.
            let _ =
                FunctionCacheRepo::add_function(cache_conn, &function_key, &addr, timeout).await;
            info!(
                "Function '{}' for user '{}' started at: {}",
                name, user_uuid, addr
            );
            Ok(addr)
        }
    }
}
