use crate::auth0_config::Auth0Config;
#[cfg(any(test, feature = "inmemory"))]
pub use crate::cache::inmemory::InMemoryCache as Cache;
#[cfg(not(any(test, feature = "inmemory")))]
pub use crate::cache::redis::RedisCache as Cache;
use crate::errors::PrimaBridgeResult;

mod inmemory;
mod redis;

pub trait Cacher {
    fn new(config: &Auth0Config) -> PrimaBridgeResult<Self>
    where
        Self: Sized;

    fn get(&mut self, key: String) -> PrimaBridgeResult<Option<String>>;
    fn set(&mut self, key: String, value: String) -> PrimaBridgeResult<()>;
}
