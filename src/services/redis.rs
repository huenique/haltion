use redis::Commands;
use std::error::Error;

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

    pub async fn set_key(&mut self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        self.con.set(key, value).map_err(|e| e.into())
    }

    pub async fn get_key(&mut self, key: &str) -> Result<String, Box<dyn Error>> {
        self.con.get(key).map_err(|e| e.into())
    }

    pub async fn del_key(&mut self, key: &str) -> Result<(), Box<dyn Error>> {
        self.con.del(key).map_err(|e| e.into())
    }
}
