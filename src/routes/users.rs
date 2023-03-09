use crate::config::env::{APP_SECRET, SMS_HOST, SMTP_HOST};
use crate::structs::AppState;
use crate::{config::env, services::users};
use axum::{extract::State, http::HeaderMap, response::IntoResponse, routing::post, Json, Router};
use reqwest::StatusCode;
use serde_json::json;

pub fn create_route() -> Router<AppState> {
    Router::new().route("/", post(store_user))
}

async fn store_user(
    headers: HeaderMap,
    State(state): State<AppState>,
    payload: Json<users::User>,
) -> impl IntoResponse {
    let mut redis = state.redis.lock().await;
    let v_result = users::verify_tenant_jwt(&headers, env::APP_SECRET.as_str()).await;
    if v_result.0 != StatusCode::OK {
        return (
            v_result.0,
            [("content-type", "application/json")],
            json!({
                "detail": v_result.1,
            })
            .to_string(),
        );
    }

    let s_result = users::store_user(&mut users::StoreUserParams {
        client: &state.http,
        db_auth: &env::DB_AUTH,
        db_url: &env::DB_URL,
        user: &users::User {
            username: payload.username.clone(),
            password: payload.password.clone(),
            client_domain: payload.client_domain.clone(),
        },
        tenant: &v_result.1,
        redis: &mut redis,
        v_host: &users::VerificationHost {
            sms: &SMS_HOST,
            smtp: &SMTP_HOST,
            smtp_port: &env::SMTP_PORT.as_str().parse::<u16>().unwrap(),
            smtp_pass: &env::SMTP_PASSWORD,
            smtp_user: &env::SMTP_USERNAME,
        },
        app_secret: &APP_SECRET,
    })
    .await;

    (
        s_result.status,
        [("content-type", "application/json")],
        json!({
            "detail": s_result.detail,
        })
        .to_string(),
    )
}
