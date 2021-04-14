use crate::auth0_configuration::Auth0Configuration;
use crate::errors::PrimaBridgeResult;
use redis::{Client, Commands, Connection, RedisError, RedisResult};

#[derive(Debug)]
pub struct Cache {
    //client: Client,
}

impl Cache {
    pub fn new(config: &Auth0Configuration) -> PrimaBridgeResult<Self> {
        //let client = redis::Client::open(config.redis_connection_uri())?;
        Ok(Self {})
    }

    pub fn read_value(&mut self, key: String) -> PrimaBridgeResult<String> {
        //let mut conn = self.client.get_connection()?;
        //Ok(conn.get(key)?)
        Ok("pippo".to_string())
    }

    pub fn set_value(&mut self, key: String, val: String) -> RedisResult<()> {
        //self.conn.set(key, val)?;
        Ok(())
    }
}
