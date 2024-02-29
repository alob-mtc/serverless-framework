use crate::function::{create_function, provision_docker, start_function, Function};
use axum::extract::Path;
use axum::routing::any;
use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use reqwest::blocking::Client;

pub async fn start_server() {
    let app = Router::new()
        .route("/upload", post(upload_function))
        .route("/function/:key", any(call_function));

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn upload_function(Json(function): Json<Function>) -> impl IntoResponse {
    println!("Received function: {:?}", function.name);
    create_function(&function.name, &function.runtime, &function.content);
    provision_docker(&function.name);
    (
        StatusCode::OK,
        format!("Function '{}' deployed successfully", function.name),
    )
}

async fn call_function(
    Path(key): Path<String>,
    Json(body): Json<serde_json::Value>, // Assuming you're using JSON for the body
) -> impl IntoResponse {
    // spawn a new thread to run the function
    let function_name = key.clone();
    println!("Starting function: {}", function_name);
    start_function(&function_name);

    println!("make a request to the server /upload");
    let client = Client::new();
    let response = client
        .post(&format!("http://localhost:8080/{key}").to_string())
        .json(&body)
        .send();

    match response {
        Ok(res) => {
            if res.status().is_success() {
                match res.text() {
                    Ok(text) => (StatusCode::OK, text),
                    Err(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to read response".to_string(),
                    ),
                }
            } else {
                (StatusCode::BAD_REQUEST, "Request failed".to_string())
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to make request".to_string(),
        ),
    }
}
