pub use inmemory::InMemoryCache;
pub use redis_impl::RedisCache;

use crate::auth0::errors::Auth0Error;
use crate::auth0::Token;

mod crypto;
mod inmemory;
mod redis_impl;

const TOKEN_PREFIX: &str = "auth0rs_tokens";
// The version of the token for backwards incompatible changes
const TOKEN_VERSION: &str = "2";

#[async_trait::async_trait]
pub trait Cache: Send + Sync + std::fmt::Debug {
    async fn get_token(&self) -> Result<Option<Token>, Auth0Error>;

    async fn put_token(&self, value_ref: &Token) -> Result<(), Auth0Error>;
}

pub(in crate::auth0::cache) fn token_key(caller: &str, audience: &str) -> String {
    format!("{}:{}:{}:{}", TOKEN_PREFIX, caller, TOKEN_VERSION, audience)
}
