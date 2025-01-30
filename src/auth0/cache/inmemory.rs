use dashmap::DashMap;

use crate::auth0::cache;
use crate::auth0::errors::Auth0Error;
use crate::auth0::token::Token;
use crate::auth0::cache::Cache;

#[derive(Clone, Debug)]
pub struct InMemoryCache {
    key_value: DashMap<String, Token>,
    caller: String,
    audience: String,
}

impl InMemoryCache {
    pub async fn new(caller: String, audience: String) -> Result<Self, Auth0Error> {
        Ok(InMemoryCache {
            key_value: DashMap::new(),
            caller,
            audience,
        })
    }
}

#[async_trait::async_trait]
impl Cache for InMemoryCache {
    async fn get_token(&self) -> Result<Option<Token>, Auth0Error> {
        let token = self
            .key_value
            .get(&cache::token_key(&self.caller, &self.audience))
            .map(|v| v.to_owned());
        Ok(token)
    }

    async fn put_token(&self, token: &Token) -> Result<(), Auth0Error> {
        let key: String = cache::token_key(&self.caller, &self.audience);
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
        let server = mockito::Server::new_async().await;
        let cache = InMemoryCache::new("caller".to_string(), "audience".to_string())
            .await
            .unwrap();

        let result: Option<Token> = cache.get_token().await.unwrap();
        assert!(result.is_none());

        let token_str: &str = "token";
        let token: Token = Token::new(token_str.to_string(), Utc::now(), Utc::now());
        cache.put_token(&token).await.unwrap();

        let result: Option<Token> = cache.get_token().await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), token_str);
    }
}
