use redis::AsyncCommands;
use serde::Deserialize;

use crate::auth0::cache::{self, crypto, Cache};
use crate::auth0::keyset::JsonWebKeySet;
use crate::auth0::token::Token;
use crate::auth0::{Auth0Error, Config};

#[derive(Clone, Debug)]
pub struct RedisCache {
    client: redis::Client,
    encryption_key: String,
    caller: String,
    audience: String,
}

impl RedisCache {
    pub async fn new(config_ref: &Config) -> Result<Self, Auth0Error> {
        let client: redis::Client = redis::Client::open(config_ref.cache_type().redis_connection_url())?;
        // Ensure connection is fine. Should fail otherwise
        let _ = client.get_async_connection().await?;

        Ok(RedisCache {
            client,
            encryption_key: config_ref.token_encryption_key().to_string(),
            caller: config_ref.caller().to_string(),
            audience: config_ref.audience().to_string(),
        })
    }

    async fn get<T>(&self, key: &str) -> Result<Option<T>, Auth0Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        self.client
            .get_async_connection()
            .await?
            .get::<_, Option<Vec<u8>>>(key)
            .await?
            .map(|value| crypto::decrypt(self.encryption_key.as_str(), value.as_slice()))
            .transpose()
    }
}

#[async_trait::async_trait]
impl Cache for RedisCache {
    async fn get_token(&self) -> Result<Option<Token>, Auth0Error> {
        let key: &str = &cache::token_key(&self.caller, &self.audience);
        self.get(key).await
    }

    async fn put_token(&self, value_ref: &Token) -> Result<(), Auth0Error> {
        let key: &str = &cache::token_key(&self.caller, &self.audience);
        let mut connection = self.client.get_async_connection().await?;
        let encrypted_value: Vec<u8> = crypto::encrypt(value_ref, self.encryption_key.as_str())?;
        let expiration: u64 = value_ref.lifetime_in_seconds();
        connection.set_ex(key, encrypted_value, expiration).await?;
        Ok(())
    }

    async fn get_jwks(&self) -> Result<Option<JsonWebKeySet>, Auth0Error> {
        let key: &str = &cache::jwks_key(&self.caller, &self.audience);
        self.get(key).await
    }

    async fn put_jwks(&self, value_ref: &JsonWebKeySet, expiration: Option<u64>) -> Result<(), Auth0Error> {
        let key: &str = &cache::jwks_key(&self.caller, &self.audience);
        let mut connection = self.client.get_async_connection().await?;
        let encrypted_value: Vec<u8> = crypto::encrypt(value_ref, self.encryption_key.as_str())?;
        let expiration: u64 = expiration.unwrap_or(86400);
        connection.set_ex(key, encrypted_value, expiration).await?;
        Ok(())
    }
}

// To run this test (it works):
// `docker run -p 6379:6379 --name bridge_redis -d redis`
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chrono::Utc;
//
//     #[tokio::test]
//     async fn redis_cache_get_set_values() {
//         let mut cache = RedisCache::new(&Config::test_config()).await.unwrap();
//
//         let result: Option<Token> = cache.get_token().await.unwrap();
//         assert!(result.is_none());
//
//         let result: Option<JsonWebKeySet> = cache.get_jwks().await.unwrap();
//         assert!(result.is_none());
//
//         let token_str: &str = "token";
//         let token: Token = Token::new(
//             token_str.to_string(),
//             Utc::now(),
//             Utc::now() + chrono::Duration::seconds(1000),
//         );
//         let () = cache.set_token(&token).await.unwrap();
//
//         let result: Option<Token> = cache.get_token().await.unwrap();
//         assert!(result.is_some());
//         assert_eq!(result.unwrap().as_str(), token_str);
//
//         let string: &str = "{\"keys\": []}";
//         let jwks: JsonWebKeySet = serde_json::from_str(string).unwrap();
//         let () = cache.set_jwks(&jwks, None).await.unwrap();
//
//         let result: Option<JsonWebKeySet> = cache.get_jwks().await.unwrap();
//         assert!(result.is_some());
//     }
// }
