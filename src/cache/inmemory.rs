use std::collections::HashMap;

use crate::auth0_configuration::Auth0CacheConfiguration;
use crate::cache::Cacher;
use crate::errors::PrimaBridgeResult;
use opentelemetry::propagation::Injector;

#[derive(Debug)]
pub struct InMemoryCache {
    key_value: HashMap<String, String>,
}

impl Cacher for InMemoryCache {
    fn new(_config: &Auth0CacheConfiguration) -> PrimaBridgeResult<Self> {
        Ok(Self {
            key_value: HashMap::new(),
        })
    }

    fn get(&mut self, key: String) -> PrimaBridgeResult<Option<String>> {
        Ok(self.key_value.get(key.as_str()).map(|v| v.to_owned()))
    }

    fn set<T: Into<String>>(&mut self, key: String, val: T) -> PrimaBridgeResult<()> {
        self.key_value.set(key.as_str(), val.into());
        Ok(())
    }
}
