use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    iss: String,
    iat: i64,
    exp: i64,
    aud: String,
    sub: String,
    scope: String,
}

impl UserClaims {
    pub async fn new(sub: String, aud: String) -> Self {
        let iat = Utc::now();
        let exp = iat + Duration::hours(24);

        Self {
            iss: env::APP_NAME.to_string(),
            aud,
            sub,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
            scope: "user".to_string(),
        }
    }
}

pub async fn sign(sub: String, aud: String) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &Header::default(),
        &UserClaims::new(sub, aud).await,
        &EncodingKey::from_secret(env::SECRET_KEY.as_bytes()),
    )
}

pub async fn verify(token: &str) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(env::SECRET_KEY.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
