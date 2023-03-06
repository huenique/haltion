use crate::config::env::APP_SECRET;
use crate::services::jwts;
use crate::structs::AppState;
use axum::{body::Body, http::Request, response::IntoResponse, routing::get, Router};
use serde_json::json;

pub fn create_route() -> Router<AppState> {
    Router::new().route("/", get(verify_jwt))
}

async fn verify_jwt(req: Request<Body>) -> impl IntoResponse {
    let headers = req.headers();
    let result = jwts::verify_jwt(headers, APP_SECRET.as_str()).await;

    (
        result.0,
        [("content-type", "application/json")],
        json!({
            "detail": result.1
        })
        .to_string(),
    )
}
