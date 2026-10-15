pub mod error;

use self::error::{RedisError, Result};
use deadpool_redis::{Config, Connection, Pool, Runtime};
use redis::{cmd, ToRedisArgs};

pub trait RedisPoolInterface {
    fn connection(&self) -> impl Future<Output = Result<impl RedisInterface>> + Send;
}

pub struct RedisPool(Pool);

impl RedisPool {
    pub fn new(redis_host: &str, password: &str, port: u16) -> Self {
        let conn_string = format!("redis://:{}@{}:{}", password, redis_host, port);
        let config = Config::from_url(conn_string);
        let pool = config.create_pool(Some(Runtime::Tokio1)).unwrap();

        Self(pool)
    }
}

impl RedisPoolInterface for RedisPool {
    async fn connection(&self) -> Result<impl RedisInterface> {
        let conn = self.0.get().await.map_err(RedisError::ConnectionError)?;
        Ok(Redis::new(conn))
    }
}

pub trait RedisInterface {
    fn set<K, V>(&mut self, key: K, value: V) -> impl Future<Output = Result<()>> + Send
    where
        K: ToRedisArgs + Send,
        V: ToRedisArgs + Send;

    fn set_ex<K, V>(
        &mut self,
        key: K,
        value: V,
        secs: usize,
    ) -> impl Future<Output = Result<()>> + Send
    where
        K: ToRedisArgs + Send,
        V: ToRedisArgs + Send;

    fn get<K: ToRedisArgs + Send>(
        &mut self,
        key: K,
    ) -> impl Future<Output = Result<Option<String>>> + Send;

    /// Gets the value and deletes the key atomically, so a value can only ever be read once.
    fn get_del<K: ToRedisArgs + Send>(
        &mut self,
        key: K,
    ) -> impl Future<Output = Result<Option<String>>> + Send;

    fn delete<K: ToRedisArgs + Send>(&mut self, key: K) -> impl Future<Output = Result<()>> + Send;
}

pub struct Redis(Connection);

impl Redis {
    fn new(connection: Connection) -> Self {
        Redis(connection)
    }
}

impl RedisInterface for Redis {
    async fn set<K, V>(&mut self, key: K, value: V) -> Result<()>
    where
        K: ToRedisArgs + Send,
        V: ToRedisArgs + Send,
    {
        cmd("SET")
            .arg(key)
            .arg(value)
            .query_async(&mut self.0)
            .await
            .map_err(RedisError::CmdError)
    }

    async fn set_ex<K, V>(&mut self, key: K, value: V, secs: usize) -> Result<()>
    where
        K: ToRedisArgs + Send,
        V: ToRedisArgs + Send,
    {
        cmd("SETEX")
            .arg(key)
            .arg(secs)
            .arg(value)
            .query_async(&mut self.0)
            .await
            .map_err(RedisError::CmdError)
    }

    async fn get<K: ToRedisArgs + Send>(&mut self, key: K) -> Result<Option<String>> {
        cmd("GET")
            .arg(&[key])
            .query_async(&mut self.0)
            .await
            .map_err(RedisError::CmdError)
    }

    async fn get_del<K: ToRedisArgs + Send>(&mut self, key: K) -> Result<Option<String>> {
        cmd("GETDEL")
            .arg(key)
            .query_async(&mut self.0)
            .await
            .map_err(RedisError::CmdError)
    }

    async fn delete<K: ToRedisArgs + Send>(&mut self, key: K) -> Result<()> {
        cmd("DEL")
            .arg(key)
            .query_async(&mut self.0)
            .await
            .map_err(RedisError::CmdError)
    }
}
