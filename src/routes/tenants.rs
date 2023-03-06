use crate::config::env;
use crate::services::tenants;
use crate::structs::AppState;
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use serde_json::json;

pub fn create_route() -> Router<AppState> {
    Router::new().route("/signup", post(create_tenant))
}

async fn create_tenant(
    State(state): State<AppState>,
    payload: Json<tenants::Tenant>,
) -> impl IntoResponse {
    let result = tenants::create_tenant(
        &state.http_client,
        &env::DB_URL,
        &env::DB_AUTH,
        &payload.name,
    )
    .await;

    (
        result.status,
        [("content-type", "application/json")],
        json!({
            "detail": result.detail,
        })
        .to_string(),
    )
}
