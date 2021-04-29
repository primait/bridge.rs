use std::collections::HashMap;

use crate::auth0_config::Auth0Config;
use crate::cache::{Cache, CacheEntry};
use crate::errors::PrimaBridgeResult;

#[derive(Clone, Debug)]
pub struct InMemoryCache {
    key_value: HashMap<String, Vec<u8>>,
    encryption_key: String,
}

impl Cache for InMemoryCache {
    fn new(config: &Auth0Config) -> PrimaBridgeResult<Self> {
        Ok(Self {
            key_value: HashMap::new(),
            encryption_key: config.token_encryption_key().to_string(),
        })
    }

    fn get(&mut self, key: &str) -> PrimaBridgeResult<Option<CacheEntry>> {
        Ok(match self.key_value.get(key) {
            Some(value) => Some(CacheEntry::decrypt(
                self.encryption_key.as_str(),
                value.to_owned(),
            )?),
            None => None,
        })
    }

    fn set(&mut self, key: &str, val: CacheEntry) -> PrimaBridgeResult<()> {
        let encrypted_value: Vec<u8> = val.encrypt(self.encryption_key.as_str())?;
        let _ = self.key_value.insert(key.to_string(), encrypted_value);
        Ok(())
    }
}

#[cfg(all(test, feature = "auth0"))]
mod tests {
    use chrono::Utc;

    use crate::auth0_config::Auth0Config;
    use crate::cache::{Cache, CacheEntry, CacheImpl};

    #[test]
    fn get_set_values() {
        let mut cache = CacheImpl::new(&Auth0Config::test_config()).unwrap();

        let cache_key: &str = "key";
        let result: Option<CacheEntry> = cache.get(cache_key).unwrap();
        assert!(result.is_none());

        let token: &str = "token";

        let entry: CacheEntry = CacheEntry::new(token.to_string(), Utc::now(), Utc::now());
        let () = cache.set(cache_key, entry).unwrap();

        let result: Option<CacheEntry> = cache.get(cache_key).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().token, token)
    }
}
