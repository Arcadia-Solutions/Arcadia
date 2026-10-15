use arcadia_storage::redis::{error::Result, RedisInterface, RedisPoolInterface};
use redis::ToRedisArgs;
#[cfg(test)]
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct MockRedisPool {
    conn: MockRedis,
}

impl MockRedisPool {
    pub fn with_conn(conn: MockRedis) -> Self {
        Self { conn }
    }
}

impl RedisPoolInterface for MockRedisPool {
    async fn connection(&self) -> Result<impl RedisInterface> {
        Ok(self.conn.clone())
    }
}

#[derive(Clone, Default)]
pub struct MockRedis {
    inner: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl RedisInterface for MockRedis {
    async fn set<K, V>(&mut self, key: K, value: V) -> Result<()>
    where
        K: ToRedisArgs + Send,
        V: ToRedisArgs + Send,
    {
        let key = key.to_redis_args()[0].clone();
        let value = value.to_redis_args()[0].clone();

        self.inner.write().unwrap().insert(key, value);
        Ok(())
    }

    async fn set_ex<K, V>(&mut self, key: K, value: V, _: usize) -> Result<()>
    where
        K: ToRedisArgs + Send,
        V: ToRedisArgs + Send,
    {
        self.set(key, value).await
    }

    async fn get<K: ToRedisArgs + Send>(&mut self, key: K) -> Result<Option<String>> {
        let key = key.to_redis_args()[0].clone();
        Ok(self
            .inner
            .read()
            .unwrap()
            .get(&key)
            .map(|v| str::from_utf8(v).unwrap().to_string()))
    }

    async fn get_del<K: ToRedisArgs + Send>(&mut self, key: K) -> Result<Option<String>> {
        let key = key.to_redis_args()[0].clone();
        Ok(self
            .inner
            .write()
            .unwrap()
            .remove(&key)
            .map(|v| str::from_utf8(&v).unwrap().to_string()))
    }

    async fn delete<K: ToRedisArgs + Send>(&mut self, key: K) -> Result<()> {
        let key = key.to_redis_args()[0].clone();
        self.inner.write().unwrap().remove(&key);
        Ok(())
    }
}
