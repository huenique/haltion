use crate::utils::jwt::TenantClaims;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug)]
pub struct SessionResult {
    pub detail: String,
    pub status: StatusCode,
}

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

pub async fn create_user(
    client: &Client,
    auth: &String,
    db_url: &String,
    user: &User,
    tenant: &String,
) -> SessionResult {
    let resp = match client
        .post(db_url)
        .header("Accept", "application/json")
        .header("Authorization", auth)
        .header("DB", tenant)
        .header("NS", tenant)
        .body(format!(
            "CREATE user CONTENT {{\"name\": \"{}\", \"password\": \"{}\"}}",
            user.username, user.password
        ))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            return SessionResult {
                detail: err.to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR,
            };
        }
    };

    SessionResult {
        status: resp.status(),
        detail: match resp.json::<String>().await {
            Ok(json) => json.to_string(),
            Err(err) => err.to_string(),
        },
    }
}

pub async fn verify_jwt(headers: &HeaderMap, secret: &'static str) -> (StatusCode, String) {
    let auth = headers.get("Authorization");
    let result = match auth {
        Some(auth_header) => {
            let v_result = verify_auth_header(auth_header, secret);
            if v_result.0 != StatusCode::OK {
                return v_result;
            }

            v_result
        }
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Authorization header is required".to_string(),
            )
        }
    };

    result
}

fn verify_auth_header(auth_header: &HeaderValue, secret: &'static str) -> (StatusCode, String) {
    let auth_header_str = auth_header.to_str().unwrap_or("");

    if !auth_header_str.starts_with("Bearer ") {
        return (
            StatusCode::BAD_REQUEST,
            "Authorization header must start with Bearer".to_string(),
        );
    }

    let token = auth_header_str.trim_start_matches("Bearer ");

    match decode::<TenantClaims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_) => (StatusCode::OK, token_.claims.tenantid),
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
    }
}
