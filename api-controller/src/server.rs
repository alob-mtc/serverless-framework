use crate::function::utils::make_request;
use crate::function::{
    function::{deploy_function, start_function, Function},
    store::FunctionStore,
};
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::{
    extract::{FromRef, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{any, post},
    Json, Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Clone, FromRef)]
struct AppState {
    function_store: FunctionStore,
}

pub async fn start_server() {
    let app_state = AppState {
        function_store: FunctionStore::new(),
    };

    let app = Router::new()
        .route("/upload", post(upload_function))
        .route("/function/:key", any(call_function))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("server listening on ... {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn upload_function(
    State(function_store): State<FunctionStore>,
    Json(function): Json<Function>,
) -> impl IntoResponse {
    println!("Received function: {:?}", function.name);
    deploy_function(&function_store, function)
        .await
        .map(|res| (StatusCode::OK, res))
}

async fn call_function(
    State(function_store): State<FunctionStore>,
    Path(key): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>, // Assuming you're using JSON for the body
) -> impl IntoResponse {
    start_function(&function_store, &key).await.map(|addr| {
        println!("making request to function: {key}");
        let res = make_request(&addr, &key, query, headers, body);
        res
    })
}
