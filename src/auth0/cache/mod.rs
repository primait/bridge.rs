#[cfg(feature = "cache-dynamodb")]
#[cfg_attr(docsrs, doc(cfg(feature = "cache-dynamodb")))]
pub use dynamodb::DynamoDBCache;

pub use inmemory::InMemoryCache;
pub use redis_impl::RedisCache;
use std::error::Error;

use crate::auth0::Token;

mod crypto;

#[cfg(feature = "cache-dynamodb")]
#[cfg_attr(docsrs, doc(cfg(feature = "cache-dynamodb")))]
pub mod dynamodb;
pub mod inmemory;
pub mod redis_impl;

const TOKEN_PREFIX: &str = "auth0rs_tokens";
// The version of the token for backwards incompatible changes
const TOKEN_VERSION: &str = "2";

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct CacheError(pub Box<dyn Error>);

#[async_trait::async_trait]
pub trait Cache: Send + Sync + std::fmt::Debug {
    async fn get_token(&self, client_id: &str, aud: &str) -> Result<Option<Token>, CacheError>;

    async fn put_token(&self, client_id: &str, aud: &str, token: &Token) -> Result<(), CacheError>;

    fn service_name(&self) -> &str;

    // The microservice name should always be prefixed, in order to simplify permission handling on
    // tools such as Redis/DynamoDB/... (permissions are usually given as "microservice:*")
    fn token_key(&self, caller: &str, audience: &str) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.service_name(),
            TOKEN_PREFIX,
            caller,
            TOKEN_VERSION,
            audience
        )
    }
}
