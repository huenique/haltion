use base64::{engine::general_purpose, Engine as _};
use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref DB_URL: String = env_or_default("DATABASE_URL");
    pub static ref DB_USERNAME: String = env_or_default("DB_USERNAME");
    pub static ref DB_PASSWORD: String = env_or_default("DB_PASSWORD");
    pub static ref SECRET_KEY: String = env_or_default("SECRET_KEY");
    pub static ref REDIS_URL: String = env_or_default("REDIS_URL");
    pub static ref SMS_HOST: String = env_or_default("SMS_HOST");
    pub static ref APP_NAME: String = env_or_default("APP_NAME");
    pub static ref DB_AUTH: String = general_purpose::STANDARD.encode(
        format!(
            "Bearer {}:{}",
            DB_USERNAME.to_string(),
            DB_PASSWORD.to_string()
        )
        .as_bytes()
    );
}

fn env_or_default(key: &str) -> String {
    match dotenvy::var(key) {
        Ok(val) => val,
        Err(_) => match env::var(key) {
            Ok(val) => val,
            Err(_) => panic!("{key} environment variable not found"),
        },
    }
}
