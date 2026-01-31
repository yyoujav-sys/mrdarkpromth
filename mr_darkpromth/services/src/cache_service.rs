use redis::{Client, Commands, RedisResult};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

pub struct CacheService {
    client: Client,
    default_ttl: Duration,
}

impl CacheService {
    pub fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            default_ttl: Duration::from_secs(3600), // 1 hour default
        })
    }

    pub fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> RedisResult<()> {
        let mut conn = self.client.get_connection()?;
        let serialized = serde_json::to_string(value).map_err(|e| {
            redis::RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string()))
        })?;

        let ttl_secs = ttl.unwrap_or(self.default_ttl).as_secs();
        let _: () = conn.set_ex(key, serialized, ttl_secs)?;
        Ok(())
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> RedisResult<Option<T>> {
        let mut conn = self.client.get_connection()?;
        let data: Option<String> = conn.get(key)?;

        match data {
            Some(s) => {
                let value = serde_json::from_str(&s).map_err(|e| {
                    redis::RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed", e.to_string()))
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub fn delete(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.client.get_connection()?;
        let _: () = conn.del(key)?;
        Ok(())
    }

    pub fn exists(&self, key: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_connection()?;
        let exists: bool = conn.exists(key)?;
        Ok(exists)
    }

    pub fn clear_pattern(&self, pattern: &str) -> RedisResult<()> {
        let mut conn = self.client.get_connection()?;
        let keys: Vec<String> = conn.keys(pattern)?;
        if !keys.is_empty() {
            let _: () = conn.del(keys)?;
        }
        Ok(())
    }
}
