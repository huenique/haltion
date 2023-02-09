use crate::routes;
use crate::services::redis::RedisClient;
use crate::structs::AppState;
use axum::Router;
use dotenv_codegen::dotenv;
use log::LevelFilter::Info;
use sea_orm::{ConnectOptions, Database};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub async fn create_app() -> Router {
    let mut opt = ConnectOptions::new(dotenv!("DATABASE_URL").to_owned());

    opt.min_connections(1)
        .max_connections(10)
        .connect_timeout(Duration::from_secs(12))
        .acquire_timeout(Duration::from_secs(12))
        .idle_timeout(Duration::from_secs(12))
        .max_lifetime(Duration::from_secs(12))
        .sqlx_logging(true)
        .sqlx_logging_level(Info);

    let db = Database::connect(opt).await.unwrap();
    let redis = Arc::new(Mutex::new(RedisClient::new().await.unwrap()));

    let state = AppState { db, redis };

    Router::new()
        .nest("/", routes::otps::create_route())
        .with_state(state)
}
