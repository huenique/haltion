use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref SECRET_KEY: String = env_or_default("SECRET_KEY");
    pub static ref REDIS_URL: String = env_or_default("REDIS_URL");
    pub static ref SMS_HOST: String = env_or_default("SMS_HOST");
    pub static ref DATABASE_URL: String = env_or_default("DATABASE_URL");
    pub static ref APP_NAME: String = env_or_default("APP_NAME");
}

fn env_or_default(key: &str) -> String {
    match dotenvy::var(key) {
        Ok(val) => val,
        Err(_) => match env::var(key) {
            Ok(val) => val,
            Err(_) => panic!("{} environment variable not found", key),
        },
    }
}
