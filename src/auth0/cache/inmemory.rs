use dashmap::DashMap;

use crate::auth0::cache::{self, crypto};
use crate::auth0::errors::Auth0Error;
use crate::auth0::keyset::JsonWebKeySet;
use crate::auth0::token::Token;
use crate::auth0::{cache::Cache, Config};

#[derive(Clone, Debug)]
pub struct InMemoryCache {
    key_value: DashMap<String, Vec<u8>>,
    encryption_key: String,
    caller: String,
    audience: String,
}

impl InMemoryCache {
    pub async fn new(config_ref: &Config) -> Result<Self, Auth0Error> {
        Ok(InMemoryCache {
            key_value: DashMap::new(),
            encryption_key: config_ref.token_encryption_key().to_string(),
            caller: config_ref.caller().to_string(),
            audience: config_ref.audience().to_string(),
        })
    }
}

#[async_trait::async_trait]
impl Cache for InMemoryCache {
    async fn get_token(&self) -> Result<Option<Token>, Auth0Error> {
        self.key_value
            .get(&cache::token_key(&self.caller, &self.audience))
            .map(|value| crypto::decrypt(self.encryption_key.as_str(), value.as_slice()))
            .transpose()
    }

    async fn put_token(&self, value_ref: &Token) -> Result<(), Auth0Error> {
        let key: String = cache::token_key(&self.caller, &self.audience);
        let encrypted_value: Vec<u8> = crypto::encrypt(value_ref, self.encryption_key.as_str())?;
        let _ = self.key_value.insert(key, encrypted_value);
        Ok(())
    }

    async fn get_jwks(&self) -> Result<Option<JsonWebKeySet>, Auth0Error> {
        self.key_value
            .get(&cache::jwks_key(&self.caller, &self.audience))
            .map(|value| crypto::decrypt(self.encryption_key.as_str(), value.as_slice()))
            .transpose()
    }

    async fn put_jwks(&self, value_ref: &JsonWebKeySet, _expiration: Option<u64>) -> Result<(), Auth0Error> {
        let key: String = cache::jwks_key(&self.caller, &self.audience);
        let encrypted_value: Vec<u8> = crypto::encrypt(value_ref, self.encryption_key.as_str())?;
        let _ = self.key_value.insert(key, encrypted_value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[tokio::test]
    async fn inmemory_cache_get_set_values() {
        let server = mockito::Server::new_async().await;
        let cache = InMemoryCache::new(&Config::test_config(&server)).await.unwrap();

        let result: Option<Token> = cache.get_token().await.unwrap();
        assert!(result.is_none());

        let result: Option<JsonWebKeySet> = cache.get_jwks().await.unwrap();
        assert!(result.is_none());

        let token_str: &str = "token";
        let token: Token = Token::new(token_str.to_string(), Utc::now(), Utc::now());
        cache.put_token(&token).await.unwrap();

        let result: Option<Token> = cache.get_token().await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), token_str);

        let string: &str = "{\"keys\": []}";
        let jwks: JsonWebKeySet = serde_json::from_str(string).unwrap();
        cache.put_jwks(&jwks, None).await.unwrap();

        let result: Option<JsonWebKeySet> = cache.get_jwks().await.unwrap();
        assert!(result.is_some());
    }
}
