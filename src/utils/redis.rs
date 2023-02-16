use redis::{aio::AsyncStream, AsyncCommands, Client};
use tokio::macros::support::Pin;

use crate::config::env::REDIS_URL;

pub struct RedisClient {
    pub client: Client,
    pub con: redis::aio::Connection<Pin<Box<dyn AsyncStream + Send + Sync>>>,
}

impl RedisClient {
    pub async fn new() -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(REDIS_URL.to_owned()).unwrap();
        let con = client.get_async_connection().await.unwrap();

        Ok(Self { client, con })
    }

    pub async fn set_key(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<Vec<String>, redis::RedisError> {
        redis::cmd("SET")
            .arg(&[key, value, "EX", "300"])
            .query_async::<_, Vec<String>>(&mut self.con)
            .await
    }

    pub async fn get_key(&mut self, key: &str) -> Result<String, redis::RedisError> {
        self.con.get(key).await
    }

    pub async fn del_key(&mut self, key: &str) -> Result<(), redis::RedisError> {
        self.con.del(key).await
    }
}
