use crate::config::constants::BEARER;
use crate::config::env::{SECRET_KEY, SMS_HOST};
use crate::services::otps;
use crate::structs::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn create_route() -> Router<AppState> {
    Router::new()
        .route("/:otp", get(verify_otp))
        .route("/", post(authorize_user))
}

async fn verify_otp(Path(otp): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let mut redis = state.redis.lock().await;
    let result = otps::verify_otp(&mut redis, &otp).await;
    let response = match result.status {
        StatusCode::OK => json!({
            "verified": true,
            "access_token": result.detail,
            "token_type": BEARER.to_string(),
        }),
        _ => json!({
            "verified": false,
            "detail": result.detail,
        }),
    };

    (
        result.status,
        [("content-type", "application/json")],
        response.to_string(),
    )
}

// TODO: Rate limit this route
#[axum_macros::debug_handler]
async fn authorize_user(
    State(state): State<AppState>,
    payload: Json<OtpPayload>,
) -> impl IntoResponse {
    let mut redis = state.redis.lock().await;
    let result = otps::authorize_user(
        &mut redis,
        &payload.phone_number,
        &payload.domain,
        &SMS_HOST,
        Client::new(),
        &SECRET_KEY,
    )
    .await;
    let resp = match result.status {
        StatusCode::OK => json!({ "sms_sent": true }),
        _ => json!({ "sms_sent": false }),
    };

    (
        StatusCode::OK,
        [("content-type", "application/json")],
        resp.to_string(),
    )
}

#[derive(Clone, Debug, Deserialize)]
pub struct OtpPayload {
    pub phone_number: String,
    pub domain: String,
}

#[derive(Debug, Serialize)]
pub struct TokenPayload {
    pub access_token: String,
    pub token_type: String,
}
