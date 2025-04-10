use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};
use uuid::Uuid;

use crate::{backend::db::auth::AuthDBRepo, rest_api::AppState};

// JWT secret key - in production, this should be loaded from an environment variable
const JWT_SECRET: &[u8] = b"your-secret-key-here";
// JWT token validity period in seconds (24 hours)
const TOKEN_VALIDITY: u64 = 24 * 60 * 60;

/// User registration request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

/// Response containing an authentication token
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    token: String,
    user: UserResponse,
}

/// Simplified user response without sensitive data
#[derive(Debug, Serialize)]
pub struct UserResponse {
    uuid: String,
    email: String,
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String, // Subject (user UUID)
    exp: u64,    // Expiration time (Unix timestamp)
    iat: u64,    // Issued at (Unix timestamp)
}

/// Handles user registration
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Validate email and password
    if payload.email.is_empty() || payload.password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Email and password are required"
            })),
        )
            .into_response();
    }

    // Check password length
    if payload.password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Password must be at least 6 characters"
            })),
        )
            .into_response();
    }

    // Register the user
    match AuthDBRepo::register(&state.db_conn, payload.email, payload.password).await {
        Ok(user) => {
            info!("User registered: {}", user.email);

            // Generate a token for the user
            match generate_token(&user.uuid.to_string()) {
                Ok(token) => {
                    let user_response = UserResponse {
                        uuid: user.uuid.to_string(),
                        email: user.email,
                    };

                    let auth_response = AuthResponse {
                        token,
                        user: user_response,
                    };

                    (StatusCode::CREATED, Json(auth_response)).into_response()
                }
                Err(e) => {
                    error!("Failed to generate token: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Failed to generate authentication token"
                        })),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            if e.to_string().contains("Email already registered") {
                return (
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "error": "Email already registered"
                    })),
                )
                    .into_response();
            }

            error!("Registration error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to register user"
                })),
            )
                .into_response()
        }
    }
}

/// Handles user login
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match AuthDBRepo::login(&state.db_conn, payload.email, payload.password).await {
        Ok(user) => {
            info!("User logged in: {}", user.email);

            // Generate a token for the user
            match generate_token(&user.uuid.to_string()) {
                Ok(token) => {
                    let user_response = UserResponse {
                        uuid: user.uuid.to_string(),
                        email: user.email,
                    };

                    let auth_response = AuthResponse {
                        token,
                        user: user_response,
                    };

                    (StatusCode::OK, Json(auth_response)).into_response()
                }
                Err(e) => {
                    error!("Failed to generate token: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Failed to generate authentication token"
                        })),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            if e.to_string().contains("Invalid credentials") {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "Invalid credentials"
                    })),
                )
                    .into_response();
            }

            error!("Login error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to authenticate user"
                })),
            )
                .into_response()
        }
    }
}

/// Validates a JWT token
pub fn validate_token(token: &str) -> Result<Uuid, jsonwebtoken::errors::Error> {
    // Decode and validate the token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )?;

    // Extract the user UUID from the subject claim
    let uuid = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| jsonwebtoken::errors::ErrorKind::InvalidSubject)?;

    Ok(uuid)
}

/// Generates a JWT token for a user
fn generate_token(user_uuid: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        sub: user_uuid.to_string(),
        exp: now + TOKEN_VALIDITY,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}
