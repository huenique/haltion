use crate::structs::AppState;
use crate::{config::env, services::sessions};
use axum::{extract::State, http::HeaderMap, response::IntoResponse, routing::post, Json, Router};
use serde_json::json;

pub fn create_route() -> Router<AppState> {
    Router::new().route("/signup", post(signup))
}

async fn signup(
    headers: HeaderMap,
    State(state): State<AppState>,
    payload: Json<sessions::User>,
) -> impl IntoResponse {
    let v_result = sessions::verify_jwt(&headers, "secret").await;
    let s_result = sessions::create_user(
        &state.http_client,
        &env::DB_AUTH.to_string(),
        &env::DB_URL,
        &sessions::User {
            username: payload.username.clone(),
            password: payload.password.clone(),
        },
        &v_result.1,
    )
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
