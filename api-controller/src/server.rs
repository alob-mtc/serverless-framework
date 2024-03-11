use crate::function::utils::make_request;
use crate::function::{
    function::{deploy_function, start_function},
    store::FunctionStore,
};
use axum::http::HeaderMap;
use axum::{
    extract::{FromRef, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{any, post},
    Json, Router,
};

use crate::function::function::Function;
use futures_util::stream::StreamExt;
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
    mut multipart: Multipart,
) -> impl IntoResponse {
    // TODO: prepare error handling
    while let Ok(Some(mut field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap().to_owned();
        if file_name.ends_with(".zip") {
            let mut buffer = Vec::new();
            while let Some(chunk) = field.next().await {
                buffer.extend_from_slice(&chunk.unwrap());
            }

            let function_name = file_name.split('.').collect::<Vec<&str>>()[0];
            println!("Received function: {:?}", function_name);
            let function = Function {
                name: function_name.to_string(),
                runtime: "go".to_string(),
                content: buffer,
            };
            return deploy_function(&function_store, function)
                .await
                .map(|res| (StatusCode::OK, res));
        }
    }
    Ok((StatusCode::BAD_REQUEST, "Unexpected req".to_string()))
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
