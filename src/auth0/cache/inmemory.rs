use std::collections::HashMap;

use crate::auth0::cache::{self, crypto};
use crate::auth0::errors::Auth0Error;
use crate::auth0::keyset::JsonWebKeySet;
use crate::auth0::token::Token;
use crate::auth0::{cache::Cache, Config};

#[derive(Clone, Debug)]
pub struct InMemoryCache {
    key_value: HashMap<String, Vec<u8>>,
    encryption_key: String,
    caller: String,
    audience: String,
}

impl Cache for InMemoryCache {
    fn new(config_ref: &Config) -> Result<Self, Auth0Error> {
        Ok(Self {
            key_value: HashMap::new(),
            encryption_key: config.token_encryption_key().to_string(),
            caller: config_ref.caller().to_string(),
            audience: config_ref.audience().to_string(),
        })
    }

    fn get_token(&mut self) -> Result<Option<Token>, Auth0Error> {
        let key: &str = &cache::token_key(&self.caller, &self.audience);
        Ok(match self.key_value.get(key) {
            Some(value) => Some(crypto::decrypt(
                self.encryption_key.as_str(),
                value.to_owned(),
            )?),
            None => None,
        })
    }

    fn set_token(&mut self, value_ref: &Token) -> Result<(), Auth0Error> {
        let key: &str = &cache::token_key(&self.caller, &self.audience);
        let encrypted_value: Vec<u8> = crypto::encrypt(value_ref, self.encryption_key.as_str())?;
        let _ = self.key_value.insert(key.to_string(), encrypted_value);
        Ok(())
    }

    fn get_jwks(&self) -> Result<Option<JsonWebKeySet>, Auth0Error> {
        let key: &str = &cache::jwks_key(&self.caller, &self.audience);
        Ok(match self.key_value.get(key) {
            Some(value) => Some(crypto::decrypt(
                self.encryption_key.as_str(),
                value.to_owned(),
            )?),
            None => None,
        })
    }

    fn set_jwks(&mut self, value_ref: &JsonWebKeySet) -> Result<(), Auth0Error> {
        let key: &str = &cache::jwks_key(&self.caller, &self.audience);
        let encrypted_value: Vec<u8> = crypto::encrypt(value_ref, self.encryption_key.as_str())?;
        let _ = self.key_value.insert(key.to_string(), encrypted_value);
        Ok(())
    }
}

mod tests {
    use chrono::Utc;

    use crate::auth0_config::Auth0Config;
    use crate::cache::{Cache, CacheEntry, CacheImpl};

    #[test]
    fn get_set_values() {
        let mut cache = CacheImpl::new(&Auth0Config::test_config()).unwrap();

        let cache_key: &str = "key";
        let result: Option<CacheEntry> = cache.get(cache_key).unwrap();
        assert!(result.is_none());

        let token: &str = "token";

        let entry: CacheEntry = CacheEntry::new(token.to_string(), Utc::now(), Utc::now());
        let () = cache.set(cache_key, entry).unwrap();

        let result: Option<CacheEntry> = cache.get(cache_key).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().token, token)
    }
}
