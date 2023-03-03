use crate::utils::jwt::UserClaims;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

pub async fn verify_jwt(headers: &HeaderMap, secret: &'static str) -> (StatusCode, &'static str) {
    let auth = headers.get("Authorization");
    let result = match auth {
        Some(auth_header) => verify_auth_header(auth_header, secret),
        None => (StatusCode::BAD_REQUEST, "Authorization header is required"),
    };

    result
}

fn verify_auth_header(
    auth_header: &HeaderValue,
    secret: &'static str,
) -> (StatusCode, &'static str) {
    let auth_header_str = auth_header.to_str().unwrap_or("");

    if !auth_header_str.starts_with("Bearer ") {
        return (
            StatusCode::BAD_REQUEST,
            "Authorization header must start with Bearer",
        );
    }

    let token = auth_header_str.trim_start_matches("Bearer ");

    match decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(_) => (StatusCode::OK, "Valid token"),
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid token"),
    }
}
