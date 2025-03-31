use crate::AppState;
use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::IntoResponse;
use futures_util::stream::StreamExt;
use service::{
    deploy_function::deploy_function,
    models::Function,
    run_function::{check_function_status, start_function},
    utils::make_request,
};
use std::collections::HashMap;
use tracing::{error, info};

/// Handles uploading a function as a ZIP file.
///
/// This endpoint expects a multipart request with one or more files.
/// If a file with a name ending in ".zip" is found, it reads its content
/// and deploys the function.
///
/// Returns an HTTP response indicating success or an appropriate error.
pub(crate) async fn upload_function(
    state: State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Iterate over the fields in the multipart request.
    while let Ok(Some(mut field)) = multipart.next_field().await {
        // Check if the field has a file name.
        if let Some(file_name) = field.file_name() {
            let file_name = file_name.to_owned();
            // Process only .zip files.
            if file_name.ends_with(".zip") {
                let mut buffer = Vec::new();
                // Read file content in chunks.
                while let Some(chunk_result) = field.next().await {
                    match chunk_result {
                        Ok(chunk) => buffer.extend_from_slice(&chunk),
                        Err(e) => {
                            error!("Error reading file chunk: {}", e);
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Error reading file")
                                .into_response();
                        }
                    }
                }

                let function_name = file_name.strip_suffix(".zip").unwrap_or(&file_name);
                info!("Received service: {}", function_name);

                let function = Function {
                    name: function_name.to_string(),
                    runtime: "go".to_string(), // TODO: Consider making runtime configurable.
                    content: buffer,
                };

                return match deploy_function(&state.db_conn, function).await {
                    Ok(res) => (StatusCode::OK, res).into_response(),
                    Err(e) => {
                        error!("Error deploying function {}: {}", function_name, e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to deploy function",
                        )
                            .into_response()
                    }
                };
            }
        } else {
            error!("Encountered a multipart field without a filename");
        }
    }
    (StatusCode::BAD_REQUEST, "Unexpected request").into_response()
}

/// Handles calling a function service based on a provided key.
///
/// This endpoint:
/// - Checks the function status in the database.
/// - Starts the function if needed (using a cache connection).
/// - Forwards the incoming request (including headers and query parameters) to the service.
///
/// Returns the service’s response or an error if any step fails.
pub(crate) async fn call_function(
    mut state: State<AppState>,
    Path(key): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    request: Request<Body>,
) -> impl IntoResponse {
    // Verify the function is in a valid state.
    if let Err(e) = check_function_status(&state.db_conn, &key).await {
        error!("Function status check failed for {}: {}", key, e);
        return e.into_response();
    }

    // Attempt to start the function using the cache connection.
    let addr = match start_function(&mut state.cache_conn, &key).await {
        Ok(addr) => addr,
        Err(e) => {
            error!("Error starting function {}: {:?}", key, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to start function",
            )
                .into_response();
        }
    };

    info!("Making request to service: {}", key);
    // Forward the request to the service and return its response.
    make_request(&addr, &key, query, headers, request)
        .await
        .into_response()
}
