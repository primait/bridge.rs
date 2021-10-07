#[cfg(test)]
pub use inmemory::InMemoryCache as CacheImpl;
#[cfg(not(test))]
pub use redis_impl::RedisCache as CacheImpl;

use crate::auth0::errors::Auth0Error;
use crate::auth0::keyset::JsonWebKeySet;
use crate::auth0::{Config, Token};

mod crypto;
#[cfg(test)]
mod inmemory;
mod redis_impl;

const TOKEN_PREFIX: &str = "auth0rs_tokens";
const JWKS_PREFIX: &str = "auth0rs_jwks";

pub trait Cache: Clone
where
    Self: Sized,
{
    fn new(config_ref: &Config) -> Result<Self, Auth0Error>
    where
        Self: Sized;

    fn get_token(&self) -> Result<Option<Token>, Auth0Error>;
    fn set_token(&mut self, value_ref: &Token) -> Result<(), Auth0Error>;
    fn get_jwks(&self) -> Result<Option<JsonWebKeySet>, Auth0Error>;
    fn set_jwks(&mut self, value_ref: &JsonWebKeySet) -> Result<(), Auth0Error>;
}

pub(in crate::auth0::cache) fn token_key(caller: &str, audience: &str) -> String {
    format!("{}:{}:{}", TOKEN_PREFIX, caller, audience)
}

pub(in crate::auth0::cache) fn jwks_key(caller: &str, audience: &str) -> String {
    format!("{}:{}:{}", JWKS_PREFIX, caller, audience)
}
