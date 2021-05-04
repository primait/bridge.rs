use redis::{Client, Commands};

use crate::auth0_config::Auth0Config;
use crate::cache::{Cache, CacheEntry};
use crate::errors::PrimaBridgeResult;

#[derive(Clone, Debug)]
pub struct RedisCache {
    client: Client,
    encryption_key: String,
}

impl Cache for RedisCache {
    fn new(config: &Auth0Config) -> PrimaBridgeResult<Self> {
        Ok(Self {
            client: redis::Client::open(config.redis_connection_uri())?,
            encryption_key: config.token_encryption_key().to_string(),
        })
    }

    fn get(&mut self, key: &str) -> PrimaBridgeResult<Option<CacheEntry>> {
        self.client
            .get_connection()?
            .get::<_, Option<Vec<u8>>>(key)?
            .map(|value| CacheEntry::decrypt(self.encryption_key.as_str(), value))
            .transpose()
    }

    fn set(&mut self, key: &str, val: CacheEntry) -> PrimaBridgeResult<()> {
        let mut connection: redis::Connection = self.client.get_connection()?;
        let encrypted_value: Vec<u8> = val.encrypt(self.encryption_key.as_str())?;
        connection.set_ex(key, encrypted_value, val.lifetime_in_seconds())?;
        Ok(())
    }
}
