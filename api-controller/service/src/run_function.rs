use crate::error::Error;
use crate::models::Function;
use crate::store::FunctionAddr;
use crate::utils::random_port;
use docker_wrapper::core::runner::runner;
use redis::aio::MultiplexedConnection;
use repository::{cache_repo::FunctionCacheRepo, db_repo::FunctionDBRepo};
use sea_orm::DbConn;
use std::time::Duration;

pub async fn check_function_status(conn: &DbConn, name: &str) -> crate::error::Result<()> {
    let function = FunctionDBRepo::find_function_by_name(conn, name).await;

    if !function.is_some() {
        return Err(Error::FunctionNotRegistered(name.to_string()));
    }

    Ok(())
}

pub async fn start_function(
    cache_conn: &mut MultiplexedConnection,
    name: &str,
) -> crate::error::Result<String> {
    match FunctionCacheRepo::get_function(cache_conn, name).await {
        Some(addr) => {
            println!("Function already running at: {}", addr);
            Ok(addr)
        }
        None => {
            let port = random_port();
            let addr = format!("localhost:{}", port);
            let timeout = 10;
            match runner(
                name,
                &format!("{port}:8080"),
                Some(Duration::from_secs(timeout)),
            )
            .await
            {
                Err(_e) => Err(Error::FunctionFailedToStart(name.to_string())),
                Ok(_) => {
                    FunctionCacheRepo::add_function(cache_conn, name, &addr, timeout).await;
                    println!("Function started at: {addr}");
                    Ok(addr)
                }
            }
        }
    }
}
