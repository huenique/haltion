use crate::services::sessions;
use crate::structs::AppState;
use axum::{extract::State, http::HeaderMap, response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;

pub fn create_route() -> Router<AppState> {
    Router::new().route("/signup", post(signup))
}

async fn signup(
    headers: HeaderMap,
    State(state): State<AppState>,
    payload: Json<Credential>,
) -> impl IntoResponse {
    let v_result = sessions::verify_jwt(&headers, "secret", &state.db).await;
    let s_result = sessions::signup(&state.db, &v_result.1, &payload.username).await;

    (
        s_result.status,
        [("content-type", "application/json")],
        json!({
            "detail": s_result.detail,
        })
        .to_string(),
    )
}

#[derive(Clone, Debug, Deserialize)]
pub struct Credential {
    pub username: String,
    pub password: String,
}
