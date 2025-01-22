//! Stuff used to provide JWT authentication via Auth0

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use reqwest::Client;
use tokio::task::JoinHandle;
use tokio::time::Interval;

pub use config::{CacheType, Config, StalenessCheckPercentage};
pub use errors::Auth0Error;
use util::ResultExt;

use crate::auth0::cache::Cache;
pub use crate::auth0::token::Token;

mod cache;
mod config;
mod errors;
mod token;
mod util;

#[derive(Debug, Clone)]
pub struct Auth0 {
    token_lock: Arc<RwLock<Token>>,
}

impl Auth0 {
    pub async fn new(client_ref: &Client, config: Config) -> Result<Self, Auth0Error> {
        let cache: Arc<dyn Cache> = if config.is_inmemory_cache() {
            Arc::new(cache::InMemoryCache::new(&config).await?)
        } else {
            Arc::new(cache::RedisCache::new(&config).await?)
        };

        let token: Token = get_token(client_ref, &cache, &config).await?;
        let token_lock: Arc<RwLock<Token>> = Arc::new(RwLock::new(token));

        start(token_lock.clone(), client_ref.clone(), cache.clone(), config).await;

        Ok(Self { token_lock })
    }

    pub fn token(&self) -> Token {
        read(&self.token_lock)
    }
}

async fn start(
    token_lock: Arc<RwLock<Token>>,
    client: Client,
    cache: Arc<dyn Cache>,
    config: Config,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker: Interval = tokio::time::interval(*config.check_interval());

        loop {
            ticker.tick().await;

            let token = match cache.get_token().await {
                Ok(Some(token)) => token,
                Ok(None) => read(&token_lock),
                Err(e) => {
                    tracing::error!("Error reading cached JWT. Reason: {:?}", e);
                    read(&token_lock)
                }
            };

            if token.needs_refresh(&config) {
                tracing::info!("Refreshing JWT and JWKS");

                match fetch_and_update_token(&client, &cache, &config).await {
                    Ok(token) => {
                        write(&token_lock, token);
                    }
                    Err(error) => tracing::error!("Failed to fetch JWT. Reason: {:?}", error),
                }
            } else if token.expire_date() > read(&token_lock).expire_date() {
                write(&token_lock, token);
            }
        }
    })
}

// Try to fetch the token from cache. If it's found return it; fetch from auth0 and put in cache otherwise
async fn get_token(client_ref: &Client, cache_ref: &Arc<dyn Cache>, config_ref: &Config) -> Result<Token, Auth0Error> {
    match cache_ref.get_token().await {
        Ok(Some(token)) => Ok(token),
        Ok(None) => fetch_and_update_token(client_ref, cache_ref, config_ref).await,
        Err(Auth0Error::CryptoError(e)) => {
            tracing::warn!("Crypto error({}) when attempting to decrypt cached token. Ignoring", e);
            fetch_and_update_token(client_ref, cache_ref, config_ref).await
        }
        Err(e) => Err(e),
    }
}

// Unconditionally fetch a new token and update the cache
async fn fetch_and_update_token(
    client_ref: &Client,
    cache_ref: &Arc<dyn Cache>,
    config_ref: &Config,
) -> Result<Token, Auth0Error> {
    let token: Token = Token::fetch(client_ref, config_ref).await?;
    let _ = cache_ref.put_token(&token).await.log_err("JWT cache set failed");

    Ok(token)
}

fn read<T: Clone>(lock_ref: &Arc<RwLock<T>>) -> T {
    let lock_guard: RwLockReadGuard<T> = lock_ref.read().unwrap_or_else(|poison_error| poison_error.into_inner());
    (*lock_guard).clone()
}

fn write<T>(lock_ref: &Arc<RwLock<T>>, token: T) {
    let mut lock_guard: RwLockWriteGuard<T> = lock_ref
        .write()
        .unwrap_or_else(|poison_error| poison_error.into_inner());
    *lock_guard = token;
}

#[cfg(test)]
mod tests {}
