use crate::auth0::errors::Auth0Error;
use crate::auth0::keyset::JsonWebKeySet;
use crate::auth0::{Config, Token};

mod crypto;
mod inmemory;
mod redis_impl;

const TOKEN_PREFIX: &str = "auth0rs_tokens";
const JWKS_PREFIX: &str = "auth0rs_jwks";
const INMEMORY: &str = "inmemory";

#[async_trait::async_trait]
pub trait Cache: Send + Sync + std::fmt::Debug {
    async fn get_token(&self) -> Result<Option<Token>, Auth0Error>;

    async fn set_token(&mut self, value_ref: &Token) -> Result<(), Auth0Error>;

    async fn get_jwks(&self) -> Result<Option<JsonWebKeySet>, Auth0Error>;

    async fn set_jwks(
        &mut self,
        value_ref: &JsonWebKeySet,
        expiration: Option<usize>,
    ) -> Result<(), Auth0Error>;
}

#[derive(Clone, Debug)]
pub struct CacheImpl {
    inmemory: bool,
    redis_cache: Option<redis_impl::RedisCache>,
    inmemory_cache: Option<inmemory::InMemoryCache>,
}

impl CacheImpl {
    pub async fn new(config_ref: &Config) -> Result<Self, Auth0Error> {
        if config_ref.redis_connection_uri().to_lowercase() == INMEMORY {
            Ok(Self {
                inmemory: true,
                redis_cache: None,
                inmemory_cache: Some(inmemory::InMemoryCache::new(config_ref).await?),
            })
        } else {
            Ok(Self {
                inmemory: false,
                redis_cache: Some(redis_impl::RedisCache::new(config_ref).await?),
                inmemory_cache: None,
            })
        }
    }
}

#[async_trait::async_trait]
impl Cache for CacheImpl {
    async fn get_token(&self) -> Result<Option<Token>, Auth0Error> {
        if self.inmemory {
            self.inmemory_cache.as_ref().unwrap().get_token().await
        } else {
            self.redis_cache.as_ref().unwrap().get_token().await
        }
    }

    async fn set_token(&mut self, value_ref: &Token) -> Result<(), Auth0Error> {
        if self.inmemory {
            match self.inmemory_cache {
                None => panic!("In memory cache is None. Logic error"),
                Some(ref mut cache) => cache.set_token(value_ref).await,
            }
        } else {
            match self.redis_cache {
                None => panic!("Redis cache is None. Logic error"),
                Some(ref mut cache) => cache.set_token(value_ref).await,
            }
        }
    }

    async fn get_jwks(&self) -> Result<Option<JsonWebKeySet>, Auth0Error> {
        if self.inmemory {
            self.inmemory_cache.as_ref().unwrap().get_jwks().await
        } else {
            self.redis_cache.as_ref().unwrap().get_jwks().await
        }
    }

    async fn set_jwks(
        &mut self,
        value_ref: &JsonWebKeySet,
        expiration: Option<usize>,
    ) -> Result<(), Auth0Error> {
        if self.inmemory {
            match self.inmemory_cache {
                None => panic!("In memory cache is None. Logic error"),
                Some(ref mut cache) => cache.set_jwks(value_ref, expiration).await,
            }
        } else {
            match self.redis_cache {
                None => panic!("Redis cache is None. Logic error"),
                Some(ref mut cache) => cache.set_jwks(value_ref, expiration).await,
            }
        }
    }
}

pub(in crate::auth0::cache) fn token_key(caller: &str, audience: &str) -> String {
    format!("{}:{}:{}", TOKEN_PREFIX, caller, audience)
}

pub(in crate::auth0::cache) fn jwks_key(caller: &str, audience: &str) -> String {
    format!("{}:{}:{}", JWKS_PREFIX, caller, audience)
}
