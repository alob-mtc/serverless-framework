use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

pub struct FunctionAddr {
    pub name: String,
    pub addr: String,
}

#[derive(Clone)]
pub struct FunctionStore {
    store: Arc<Mutex<HashMap<String, ()>>>,
    cache: Arc<Mutex<HashMap<String, (String, Instant)>>>,
}

impl FunctionStore {
    pub fn new() -> Self {
        FunctionStore {
            store: Arc::new(Mutex::new(HashMap::new())),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// function store
impl FunctionStore {
    pub async fn register_function(&self, name: &str) {
        let mut store = self.store.lock().await;
        store.insert(name.to_string(), ());
    }

    pub async fn function_exists(&self, name: &str) -> bool {
        let store = self.store.lock().await;
        store.contains_key(name)
    }
}

// function cache
impl FunctionStore {
    pub async fn add_function(&self, function: FunctionAddr, ttl: Duration) {
        let mut store = self.cache.lock().await;
        let expiration = Instant::now() + ttl;
        store.insert(function.name, (function.addr, expiration));
    }

    pub async fn get_function(&self, name: &str) -> Option<String> {
        let mut store = self.cache.lock().await;
        if let Some((addr, expiration)) = store.get(name) {
            if *expiration > Instant::now() {
                return Some(addr.to_string());
            }
            store.remove(name);
        }
        None
    }
}
