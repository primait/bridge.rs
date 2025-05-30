use dashmap::DashMap;

use crate::auth0::cache::Cache;
use crate::auth0::token::Token;

use super::CacheError;

#[derive(Default, Clone, Debug)]
pub struct InMemoryCache {
    key_value: DashMap<String, Token>,
    service_name: String,
}

impl InMemoryCache {
    pub fn new(service_name: String) -> Self {
        Self {
            key_value: DashMap::default(),
            service_name,
        }
    }
}

#[async_trait::async_trait]
impl Cache for InMemoryCache {
    async fn get_token(&self, client_id: &str, aud: &str) -> Result<Option<Token>, CacheError> {
        let key = super::token_key(&self.service_name, client_id, aud);
        let token = self.key_value.get(key.as_str()).map(|v| v.to_owned());
        Ok(token)
    }

    async fn put_token(&self, client_id: &str, aud: &str, token: &Token) -> Result<(), CacheError> {
        let key = super::token_key(&self.service_name, client_id, aud);
        let _ = self.key_value.insert(key, token.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[tokio::test]
    async fn inmemory_cache_get_set_values() {
        let client_id = "caller".to_string();
        let audience = "audience".to_string();
        let service = "service".to_string();

        let cache = InMemoryCache::new(service);

        let result: Option<Token> = cache.get_token(&client_id, &audience).await.unwrap();
        assert!(result.is_none());

        let token_str: &str = "token";
        let token: Token = Token::new(token_str.to_string(), Utc::now(), Utc::now());
        cache.put_token(&client_id, &audience, &token).await.unwrap();

        let result: Option<Token> = cache.get_token(&client_id, &audience).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), token_str);
    }
}
