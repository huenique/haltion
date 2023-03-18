use redis::{
    aio::{AsyncStream, Connection},
    AsyncCommands, Client,
};
use tokio::macros::support::Pin;

pub struct RedisClient {
    pub client: Client,
    pub con: Connection<Pin<Box<dyn AsyncStream + Send + Sync>>>,
}

impl RedisClient {
    pub async fn new(redis_url: String) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url).unwrap();
        let con = client.get_async_connection().await.unwrap();

        Ok(Self { client, con })
    }

    pub async fn set_key(&mut self, key: &str, value: &str) -> Result<String, redis::RedisError> {
        redis::cmd("SET")
            .arg(&[key, value, "EX", "300"])
            .query_async::<_, String>(&mut self.con)
            .await
    }

    pub async fn set_key_map(
        &mut self,
        key: &str,
        items: &[(String, String)],
    ) -> Result<String, redis::RedisError> {
        redis::cmd("HMSET")
            .arg(key)
            .arg(items)
            .arg(&["EX", "300"])
            .query_async::<_, String>(&mut self.con)
            .await
    }

    pub async fn get_key(&mut self, key: &str) -> Result<String, redis::RedisError> {
        self.con.get(key).await
    }

    pub async fn del_key(&mut self, key: &str) -> Result<(), redis::RedisError> {
        self.con.del(key).await
    }
}
