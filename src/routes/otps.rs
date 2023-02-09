use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::constants;
use crate::services::jwt;
use crate::structs::AppState;

pub fn create_route() -> Router<AppState> {
    Router::new()
        .route("/otps/:otp", get(verify))
        .route("/otps", post(authorize))
}

async fn verify(
    Path(otp): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<TokenPayload>, AuthError> {
    let mut redis = state.redis.lock().await;
    let phone_number = redis.get_key(&otp).await.unwrap();
    let token = jwt::sign(phone_number.to_string()).await.unwrap();

    redis.del_key(&otp).await.unwrap();

    Ok(Json(TokenPayload {
        access_token: token,
        token_type: constants::BEARER.to_string(),
    }))
}

// TODO: Rate limit this route
#[axum_macros::debug_handler]
async fn authorize(
    State(state): State<AppState>,
    payload: Json<OtpPayload>,
) -> Result<Json<TokenPayload>, AuthError> {
    let mut redis = state.redis.lock().await;

    let token = jwt::sign(payload.phone_number.to_owned()).await?;

    let phone_u64 = payload.phone_number.parse::<u64>().unwrap();

    // redis.set_key(&token, &phone_u64).await?;

    Ok(Json(TokenPayload {
        access_token: token,
        token_type: constants::BEARER.to_string(),
    }))
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        AuthError::JwtError(error)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            AuthError::JwtError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "JWT error"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OtpPayload {
    pub phone_number: String,
}

#[derive(Debug, Serialize)]
pub struct TokenPayload {
    pub access_token: String,
    pub token_type: String,
}

#[allow(dead_code)]
#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    JwtError(jsonwebtoken::errors::Error),
}
