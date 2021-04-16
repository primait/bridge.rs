use redis::{Client, Commands};

use crate::auth0_config::Auth0Config;
use crate::cache::{CacheEntry, Cacher};
use crate::errors::PrimaBridgeResult;

#[derive(Debug)]
pub struct RedisCache {
    client: Client,
    encryption_key: String,
}

impl Cacher for RedisCache {
    fn new(config: &Auth0Config) -> PrimaBridgeResult<Self> {
        Ok(Self {
            client: redis::Client::open(config.redis_connection_uri())?,
            encryption_key: config.token_encryption_key().to_string(),
        })
    }

    fn get(&mut self, key: &str) -> PrimaBridgeResult<Option<CacheEntry>> {
        let mut connection: redis::Connection = self.client.get_connection()?;
        let value_opt: Option<Vec<u8>> = connection.get(key)?;
        Ok(match value_opt {
            Some(value) => Some(CacheEntry::decrypt(self.encryption_key.as_str(), value)?),
            None => None,
        })
    }

    fn set(&mut self, key: &str, val: CacheEntry) -> PrimaBridgeResult<()> {
        let mut connection: redis::Connection = self.client.get_connection()?;
        let encrypted_value: Vec<u8> = val.encrypt(self.encryption_key.as_str())?;
        connection.set(key, encrypted_value)?;
        Ok(())
    }
}
