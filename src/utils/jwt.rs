use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::env;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iat: i64,
    exp: i64,
}

impl Claims {
    pub async fn new(phone_number: String) -> Self {
        let iat = Utc::now();
        let exp = iat + Duration::hours(24);

        Self {
            sub: phone_number,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

pub async fn sign(phone_number: String) -> Result<String, jsonwebtoken::errors::Error> {
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(phone_number).await,
        &EncodingKey::from_secret(env::SECRET_KEY.as_bytes()),
    )?)
}

pub async fn verify(token: &str) -> Result<String, jsonwebtoken::errors::Error> {
    Ok(jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(env::SECRET_KEY.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)?)
}
