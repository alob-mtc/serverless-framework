use crate::AppState;
use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::IntoResponse;
use futures_util::StreamExt;
use service::{
    deploy_function::deploy_function,
    models::Function,
    run_function::{check_function_status, start_function},
    utils::make_request,
};
use std::collections::HashMap;

pub(crate) async fn upload_function(
    state: State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Ok(Some(mut field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap().to_owned();
        if file_name.ends_with(".zip") {
            let mut buffer = Vec::new();
            while let Some(chunk) = field.next().await {
                buffer.extend_from_slice(&chunk.unwrap());
            }

            let function_name = file_name.split('.').collect::<Vec<&str>>()[0];
            println!("Received service: {:?}", function_name);
            let function = Function {
                name: function_name.to_string(),
                runtime: "go".to_string(),
                content: buffer,
            };
            return deploy_function(&state.db_conn, function)
                .await
                .map(|res| (StatusCode::OK, res));
        }
    }
    Ok((StatusCode::BAD_REQUEST, "Unexpected req".to_string()))
}

pub(crate) async fn call_function(
    mut state: State<AppState>,
    Path(key): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    request: Request<Body>,
) -> impl IntoResponse {
    if let Err(e) = check_function_status(&state.db_conn, &key).await {
        return e.into_response()
    }

    let addr = match start_function(&mut state.cache_conn, &key).await {
        Ok(addr) => addr,
        Err(e) => {
            // Handle your error (log it, map it to an HTTP status, etc.)
            eprintln!("Error starting function: {e:?}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to start function",
            )
                .into_response();
        }
    };
    println!("Making request to service: {key}");
    make_request(&addr, &key, query, headers, request).await.into_response()
}
