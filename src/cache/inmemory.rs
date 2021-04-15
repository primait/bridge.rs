use std::collections::HashMap;

use crate::auth0_config::Auth0Config;
use crate::cache::Cacher;
use crate::errors::PrimaBridgeResult;

#[derive(Debug)]
pub struct InMemoryCache {
    key_value: HashMap<String, String>,
}

impl Cacher for InMemoryCache {
    fn new(_config: &Auth0Config) -> PrimaBridgeResult<Self> {
        Ok(Self {
            key_value: HashMap::new(),
        })
    }

    fn get(&mut self, key: String) -> PrimaBridgeResult<Option<String>> {
        Ok(self.key_value.get(key.as_str()).map(|v| v.to_owned()))
    }

    fn set(&mut self, key: String, val: String) -> PrimaBridgeResult<()> {
        let _ = self.key_value.insert(key, val);
        Ok(())
    }
}
