use crate::utils::redis::RedisClient;
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub(crate) redis: Arc<Mutex<RedisClient>>,
    pub(crate) http_client: reqwest::Client,
}
