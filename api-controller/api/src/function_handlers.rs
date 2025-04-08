use crate::AppState;
use crate::AuthenticatedUser;
use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, HeaderMap, Request, StatusCode};
use axum::response::IntoResponse;
use entity::function::Model as FunctionModel;
use futures_util::stream::StreamExt;
use repository::function_repo::FunctionDBRepo;
use service::{
    deploy_function::deploy_function,
    models::Function,
    run_function::{check_function_status, start_function},
    utils::make_request,
};
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

/// Handles uploading a function as a ZIP file with authentication.
///
/// This endpoint expects a multipart request with one or more files and an Authorization header.
/// If a file with a name ending in ".zip" is found, it reads its content
/// and deploys the function for the authenticated user.
///
/// Returns an HTTP response indicating success or an appropriate error.
pub(crate) async fn upload_function_authenticated(
    State(state): State<AppState>,
    AuthenticatedUser(user_uuid): AuthenticatedUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Get configuration from state
    let supported_archive_ext = ".zip"; // Currently we only support ZIP
    let default_runtime = &state.config.function.default_runtime;
    let max_size = state.config.function.max_function_size;

    // Iterate over the fields in the multipart request.
    while let Ok(Some(mut field)) = multipart.next_field().await {
        // Check if the field has a file name.
        if let Some(file_name) = field.file_name() {
            let file_name = file_name.to_owned();
            // Process only archive files.
            if file_name.ends_with(supported_archive_ext) {
                // Read file content in chunks.
                let buffer = match read_field_chunks(&mut field, max_size).await {
                    Ok(buffer) => buffer,
                    Err(e) => {
                        error!("Error reading file chunk: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Error reading file: {}", e),
                        )
                            .into_response();
                    }
                };

                let function_name = file_name
                    .strip_suffix(supported_archive_ext)
                    .unwrap_or(&file_name);
                info!("Received service: {}", function_name);

                let function = Function {
                    name: function_name.to_string(),
                    runtime: default_runtime.clone(),
                    content: buffer,
                    user_uuid,
                };

                // Deploy the function
                return match deploy_function(&state.db_conn, function).await {
                    Ok(res) => (
                        StatusCode::OK,
                        format!(
                            "{}\nFunction: {}\nUser UUID: {}",
                            res, function_name, user_uuid
                        ),
                    )
                        .into_response(),
                    Err(e) => {
                        error!("Error deploying function {}: {}", function_name, e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to deploy function: {}", e),
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

/// List functions for an authenticated user
pub(crate) async fn list_functions(
    State(state): State<AppState>,
    AuthenticatedUser(user_uuid): AuthenticatedUser,
) -> impl IntoResponse {
    // Get functions for this user
    match FunctionDBRepo::find_functions_by_user_uuid(&state.db_conn, user_uuid).await {
        Ok(functions) => {
            // Convert to a simpler representation
            let function_list = functions
                .into_iter()
                .map(|f| {
                    serde_json::json!({
                        "uuid": f.uuid.to_string(),
                        "name": f.name,
                        "runtime": f.runtime
                    })
                })
                .collect::<Vec<_>>();

            (StatusCode::OK, axum::Json(function_list)).into_response()
        }
        Err(e) => {
            error!("Error listing functions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error listing functions: {}", e),
            )
                .into_response()
        }
    }
}

/// Reads all chunks from a multipart field into a buffer.
async fn read_field_chunks(
    field: &mut axum::extract::multipart::Field<'_>,
    max_size: usize,
) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    let mut total_size = 0;

    while let Some(chunk_result) = field.next().await {
        match chunk_result {
            Ok(chunk) => {
                total_size += chunk.len();
                if total_size > max_size {
                    return Err(format!(
                        "File too large, maximum size is {} bytes",
                        max_size
                    ));
                }
                buffer.extend_from_slice(&chunk);
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(buffer)
}

/// Handles calling a function service based on a provided key.
///
/// This endpoint:
/// - Checks the function status in the database.
/// - Starts the function if needed (using a cache connection).
/// - Forwards the incoming request (including headers and query parameters) to the service.
///
/// Returns the service's response or an error if any step fails.
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
                format!("Failed to start function: {}", e),
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
