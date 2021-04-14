use crate::auth0_configuration::Auth0CacheConfiguration;
use crate::cache::redis::RedisCache;
use crate::errors::PrimaBridgeResult;
use inmemory::InMemoryCache;

mod inmemory;
mod redis;

#[cfg(any(test, feature = "inmemory"))]
pub type Cache = InMemoryCache;

#[cfg(not(any(test, feature = "inmemory")))]
pub type Cache = RedisCache;

pub trait Cacher {
    fn new(config: &Auth0CacheConfiguration) -> PrimaBridgeResult<Self>
    where
        Self: Sized;

    fn get(&mut self, key: String) -> PrimaBridgeResult<Option<String>>;
    fn set<T: Into<String>>(&mut self, key: String, value: T) -> PrimaBridgeResult<()>;
}
