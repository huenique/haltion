use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::constants::BEARER;
use crate::utils::jwt;
use crate::services::otps;
use crate::structs::AppState;

pub fn create_route() -> Router<AppState> {
    Router::new()
        .route("/otps/:otp", get(verify))
        .route("/otps", post(authorize))
}

async fn verify(Path(otp): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let mut redis = state.redis.lock().await;

    let phone_number = match otps::verify(&mut redis, &otp).await {
        Ok(phone_number) => phone_number,
        Err(err) => {
            return (
                err.status,
                [("content-type", "application/json")],
                json!({
                    "detail": err.detail
                })
                .to_string(),
            )
        }
    };

    let token = jwt::sign(phone_number).await.unwrap();

    redis.del_key(&otp).await.unwrap();

    (
        StatusCode::OK,
        [("content-type", "application/json")],
        json!({
            "access_token": token,
            "token_type": BEARER.to_string()
        })
        .to_string(),
    )
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
