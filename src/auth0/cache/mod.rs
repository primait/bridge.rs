pub use inmemory::InMemoryCache;
pub use redis_impl::RedisCache;

use crate::auth0::errors::Auth0Error;
use crate::auth0::keyset::JsonWebKeySet;
use crate::auth0::Token;

mod crypto;
mod inmemory;
mod redis_impl;

const TOKEN_PREFIX: &str = "auth0rs_tokens";
const JWKS_PREFIX: &str = "auth0rs_jwks";

#[async_trait::async_trait]
pub trait Cache: Send + Sync + std::fmt::Debug {
    async fn get_token(&self) -> Result<Option<Token>, Auth0Error>;

    async fn put_token(&self, value_ref: &Token) -> Result<(), Auth0Error>;

    async fn get_jwks(&self) -> Result<Option<JsonWebKeySet>, Auth0Error>;

    async fn put_jwks(
        &self,
        value_ref: &JsonWebKeySet,
        expiration: Option<usize>,
    ) -> Result<(), Auth0Error>;
}

pub(in crate::auth0::cache) fn token_key(caller: &str, audience: &str) -> String {
    format!("{}:{}:{}", TOKEN_PREFIX, caller, audience)
}

pub(in crate::auth0::cache) fn jwks_key(caller: &str, audience: &str) -> String {
    format!("{}:{}:{}", JWKS_PREFIX, caller, audience)
}
