use redis::Commands;
use std::error::Error;

pub struct RedisClient {
    con: redis::Connection,
}

impl RedisClient {
    pub async fn new() -> Result<Self, redis::RedisError> {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
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
