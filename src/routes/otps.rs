use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

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
) -> Result<Json<TokenPayload>, StatusCode> {
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
) -> Result<Json<AuthResult>, StatusCode> {
    let mut redis = state.redis.lock().await;
    let token = jwt::sign(payload.phone_number.to_owned()).await.unwrap();

    redis.set_key(&token, &payload.phone_number).await.unwrap();

    // TODO: Send OTP via SMS
    // ...

    Ok(Json(AuthResult { sms_sent: true }))
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

#[derive(Debug, Serialize)]
pub struct AuthResult {
    pub sms_sent: bool,
}
