use redis::AsyncCommands;
use serde::Deserialize;

use crate::auth0::cache::{self, crypto, Cache};
use crate::auth0::token::Token;
use crate::auth0::Auth0Error;

#[derive(Clone, Debug)]
pub struct RedisCache {
    client: redis::Client,
    encryption_key: String,
}

impl RedisCache {
    /// Redis connection string(eg. `"redis://{host}:{port}?{ParamKey1}={ParamKey2}"`)
    pub async fn new(redis_connection_url: String, token_encryption_key: String) -> Result<Self, Auth0Error> {
        let client: redis::Client = redis::Client::open(redis_connection_url)?;
        // Ensure connection is fine. Should fail otherwise
        let _ = client.get_multiplexed_async_connection().await?;

        Ok(RedisCache {
            client,
            encryption_key: token_encryption_key,
        })
    }

    async fn get<T>(&self, key: &str) -> Result<Option<T>, Auth0Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        self.client
            .get_multiplexed_async_connection()
            .await?
            .get::<_, Option<Vec<u8>>>(key)
            .await?
            .map(|value| crypto::decrypt(self.encryption_key.as_str(), value.as_slice()))
            .transpose()
    }
}

#[async_trait::async_trait]
impl Cache for RedisCache {
    async fn get_token(&self, client_id: &str, audience: &str) -> Result<Option<Token>, Auth0Error> {
        let key: &str = &cache::token_key(client_id, audience);
        self.get(key).await
    }

    async fn put_token(&self, client_id: &str, audience: &str, value_ref: &Token) -> Result<(), Auth0Error> {
        let key: &str = &cache::token_key(client_id, audience);

        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let encrypted_value: Vec<u8> = crypto::encrypt(value_ref, self.encryption_key.as_str())?;
        let expiration = value_ref.lifetime_in_seconds();
        let _: () = connection.set_ex(key, encrypted_value, expiration).await?;
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
