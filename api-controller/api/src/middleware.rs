use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

use crate::{auth_handlers::validate_token, AppState};

/// Extractor for authenticated user UUID
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub Uuid);

/// Error response for authentication failures
#[derive(Debug)]
pub struct AuthError(pub StatusCode, pub String);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let AuthError(status, message) = self;
        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// Authentication middleware that extracts the user UUID from the JWT token
#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the authorization header
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                AuthError(
                    StatusCode::UNAUTHORIZED,
                    "Missing authorization header".to_string(),
                )
            })?;

        // Check if the authorization header starts with "Bearer "
        if !auth_header.starts_with("Bearer ") {
            return Err(AuthError(
                StatusCode::UNAUTHORIZED,
                "Invalid authorization header format".to_string(),
            ));
        }

        // Extract the token
        let token = &auth_header[7..];

        // Validate the token
        let user_uuid = validate_token(token).map_err(|e| {
            error!("Token validation error: {}", e);
            AuthError(
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token".to_string(),
            )
        })?;

        // Get the app state
        let app_state = AppState::from_ref(state);

        // Verify the user exists in the database
        match repository::auth_repo::AuthDBRepo::find_by_uuid(&app_state.db_conn, user_uuid).await {
            Ok(Some(_)) => Ok(AuthenticatedUser(user_uuid)),
            Ok(None) => Err(AuthError(
                StatusCode::UNAUTHORIZED,
                "User not found".to_string(),
            )),
            Err(e) => {
                error!("Error finding user by UUID: {}", e);
                Err(AuthError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                ))
            }
        }
    }
}
