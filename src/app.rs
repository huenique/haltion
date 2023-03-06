use crate::config::env::REDIS_URL;
use crate::routes;
use crate::structs::AppState;
use crate::utils::redis::RedisClient;
use axum::Router;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn create_app() -> Router {
    let redis = Arc::new(Mutex::new(
        RedisClient::new(REDIS_URL.to_owned()).await.unwrap(),
    ));
    let http_client = Client::new();
    let state = AppState { redis, http_client };

    Router::new()
        .nest("/otps", routes::otps::create_route())
        .nest("/jwts", routes::jwts::create_route())
        .nest("/tenants", routes::tenants::create_route())
        .nest("/users", routes::users::create_route())
        .with_state(state)
}
