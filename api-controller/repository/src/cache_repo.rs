use redis::{aio::MultiplexedConnection, AsyncCommands, ExistenceCheck, SetExpiry, SetOptions};

pub struct FunctionCacheRepo;

impl FunctionCacheRepo {
    pub async fn get_function(conn: &mut MultiplexedConnection, name: &str) -> Option<String> {
        conn.get(name).await.unwrap()
    }
    pub async fn add_function(conn: &mut MultiplexedConnection, name: &str, addr: &str, ttl: u64) {
        let opts = SetOptions::default()
            .conditional_set(ExistenceCheck::NX)
            .get(true)
            .with_expiration(SetExpiry::EX(ttl));
        conn.set_options(name, addr, opts).await.unwrap()
    }
}
