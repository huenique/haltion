use redis::Commands;

use crate::config::env::REDIS_URL;

pub struct RedisClient {
    con: redis::Connection,
}

impl RedisClient {
    pub async fn new() -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(REDIS_URL.to_owned()).unwrap();
        let con = client.get_connection()?;

        Ok(Self { con })
    }

    pub async fn set_key(&mut self, key: &str, value: &str) -> Result<(), redis::RedisError> {
        self.con.set(key, value)
    }

    pub async fn get_key(&mut self, key: &str) -> Result<String, redis::RedisError> {
        self.con.get(key)
    }

    pub async fn del_key(&mut self, key: &str) -> Result<(), redis::RedisError> {
        self.con.del(key)
    }
}
