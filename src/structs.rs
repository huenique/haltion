use crate::utils::redis::RedisClient;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub(crate) db: DatabaseConnection,
    pub(crate) redis: Arc<Mutex<RedisClient>>,
}
