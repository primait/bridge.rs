use redis::{Client, Commands};

use crate::auth0_config::Auth0Config;
use crate::cache::Cacher;
use crate::errors::PrimaBridgeResult;

#[derive(Debug)]
pub struct RedisCache {
    client: Client,
}

impl Cacher for RedisCache {
    fn new(config: &Auth0Config) -> PrimaBridgeResult<Self> {
        Ok(Self {
            client: redis::Client::open(config.redis_connection_uri())?,
        })
    }

    fn get(&mut self, key: String) -> PrimaBridgeResult<Option<String>> {
        let mut conn = self.client.get_connection()?;
        Ok(conn.get(key)?)
    }

    fn set(&mut self, key: String, val: String) -> PrimaBridgeResult<()> {
        let _ = self.client.get_connection()?.set(key, val)?;
        Ok(())
    }
}
