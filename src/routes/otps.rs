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
        .route("/otps/:otp", get(verify))
        .route("/otps", post(authorize))
}

async fn verify(Path(otp): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let mut redis = state.redis.lock().await;

    let token = match otps::verify(&mut redis, &otp).await {
        Ok(phone_number) => phone_number,
        Err(err) => {
            return (
                err.status,
                [("content-type", "application/json")],
                json!({
                    "verified": false,
                    "detail": err.detail
                })
                .to_string(),
            )
        }
    };

    (
        StatusCode::OK,
        [("content-type", "application/json")],
        json!({
            "verified": true,
            "access_token": token,
            "token_type": BEARER.to_string()
        })
        .to_string(),
    )
}

// TODO: Rate limit this route
#[axum_macros::debug_handler]
async fn authorize(State(state): State<AppState>, payload: Json<OtpPayload>) -> impl IntoResponse {
    let mut redis = state.redis.lock().await;

    match otps::authorize(
        &mut redis,
        &payload.phone_number,
        &SMS_HOST,
        Client::new(),
        &SECRET_KEY,
    )
    .await
    {
        Ok(_) => (),
        Err(err) => {
            return (
                err.status,
                [("content-type", "application/json")],
                json!({
                    "sms_sent": false,
                    "detail": err.detail
                })
                .to_string(),
            )
        }
    };

    (
        StatusCode::OK,
        [("content-type", "application/json")],
        json!({
            "sms_sent": true
        })
        .to_string(),
    )
}

#[derive(Clone, Debug, Deserialize)]
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
