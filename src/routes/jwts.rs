use crate::config::env::SECRET_KEY;
use crate::structs::AppState;
use crate::utils::jwt::Claims;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::json;

pub fn create_route() -> Router<AppState> {
    Router::new().route("/", get(verify_jwt))
}

async fn verify_jwt(req: Request<Body>) -> impl IntoResponse {
    let headers = req.headers();
    let authorization = headers.get("Authorization");

    let response_json = match authorization {
        Some(auth_header) => {
            let auth_header = auth_header.to_str().unwrap_or("");
            if !auth_header.starts_with("Bearer ") {
                (
                    StatusCode::BAD_REQUEST,
                    "Authorization header must start with Bearer",
                )
            } else {
                let token = auth_header.trim_start_matches("Bearer ");
                match decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(SECRET_KEY.as_ref()),
                    &Validation::new(Algorithm::HS256),
                ) {
                    Ok(_) => (StatusCode::OK, "Valid token"),
                    Err(_) => (StatusCode::UNAUTHORIZED, "Invalid token"),
                }
            }
        }
        None => (StatusCode::BAD_REQUEST, "Authorization header is required"),
    };

    (
        response_json.0,
        [("content-type", "application/json")],
        json!({
            "detail": response_json.1
        })
        .to_string(),
    )
}
