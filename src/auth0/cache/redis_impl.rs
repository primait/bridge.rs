use redis::Commands;

use crate::auth0::cache::{self, crypto, Cache};
use crate::auth0::errors::Auth0Error;
use crate::auth0::keyset::JsonWebKeySet;
use crate::auth0::token::Token;
use crate::auth0::Config;

#[derive(Clone, Debug)]
pub struct RedisCache {
    client: redis::Client,
    encryption_key: String,
    caller: String,
    audience: String,
}

impl Cache for RedisCache {
    fn new(config_ref: &Config) -> Result<Self, Auth0Error> {
        let client: redis::Client = redis::Client::open(config_ref.redis_connection_uri())?;
        // Ensure connection is fine. Should fail otherwise
        let _: redis::Connection = client.get_connection()?;

        Ok(Self {
            client,
            encryption_key: config_ref.token_encryption_key().to_string(),
            caller: config_ref.caller().to_string(),
            audience: config_ref.audience().to_string(),
        })
    }

    fn get_token(&self) -> Result<Option<Token>, Auth0Error> {
        let key: &str = &cache::token_key(&self.caller, &self.audience);
        self.client
            .get_connection()?
            .get::<_, Option<Vec<u8>>>(key)?
            .map(|value| crypto::decrypt(self.encryption_key.as_str(), value))
            .transpose()
    }

    fn set_token(&mut self, value_ref: &Token) -> Result<(), Auth0Error> {
        let key: &str = &cache::token_key(&self.caller, &self.audience);
        let mut connection: redis::Connection = self.client.get_connection()?;
        let encrypted_value: Vec<u8> = crypto::encrypt(value_ref, self.encryption_key.as_str())?;
        let expiration: usize = value_ref.lifetime_in_seconds();
        connection.set_ex(key, encrypted_value, expiration)?;
        Ok(())
    }

    fn get_jwks(&self) -> Result<Option<JsonWebKeySet>, Auth0Error> {
        let key: &str = &cache::jwks_key(&self.caller, &self.audience);
        self.client
            .get_connection()?
            .get::<_, Option<Vec<u8>>>(key)?
            .map(|value| crypto::decrypt(self.encryption_key.as_str(), value))
            .transpose()
    }

    fn set_jwks(&mut self, value_ref: &JsonWebKeySet) -> Result<(), Auth0Error> {
        let key: &str = &cache::jwks_key(&self.caller, &self.audience);
        let mut connection: redis::Connection = self.client.get_connection()?;
        let encrypted_value: Vec<u8> = crypto::encrypt(value_ref, self.encryption_key.as_str())?;
        // FIXME: take expiration from http headers of jwks call
        let expiration: usize = 84000;
        connection.set_ex(key, encrypted_value, expiration)?;
        Ok(())
    }
}
