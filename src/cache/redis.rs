use redis::Client;

use crate::auth0_configuration::{Auth0CacheConfiguration, Auth0Configuration};
use crate::cache::Cacher;
use crate::errors::PrimaBridgeResult;

#[derive(Debug)]
pub struct RedisCache {
    client: Client,
}

impl Cacher for RedisCache {
    fn new(config: &Auth0CacheConfiguration) -> PrimaBridgeResult<Self> {
        let client = redis::Client::open(config.redis_connection_uri())?;
        Ok(Self { client })
    }

    fn get(&mut self, key: String) -> PrimaBridgeResult<Option<String>> {
        //let mut conn = self.client.get_connection()?;
        //Ok(conn.get(key)?)
        Ok(Some("pippo".to_string()))
    }

    fn set<T: Into<String>>(&mut self, key: String, val: T) -> PrimaBridgeResult<()> {
        //self.conn.set(key, val)?;
        Ok(())
    }
}
