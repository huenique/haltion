use crate::services::tenants;
use crate::structs::AppState;
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;

pub fn create_route() -> Router<AppState> {
    Router::new().route("/signup", post(signup))
}

// The payload should contain the name of the tenant, which we will use to
// create the tenant's schema.
async fn signup(State(state): State<AppState>, payload: Json<Tenant>) -> impl IntoResponse {
    let result = tenants::signup(&state.db, &payload.name).await;

    (
        result.0,
        [("content-type", "application/json")],
        json!({
            "detail": result.1
        })
        .to_string(),
    )
}

#[derive(Clone, Debug, Deserialize)]
pub struct Tenant {
    pub name: String,
}
