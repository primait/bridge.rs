//! Stuff used to provide JWT authentication via Auth0

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use reqwest::Client;
use tokio::task::JoinHandle;
use tokio::time::Interval;

pub use config::{CacheType, Config, StalenessCheckPercentage};
pub use errors::Auth0Error;
use util::ResultExt;

use crate::auth0::cache::Cache;
use crate::auth0::keyset::JsonWebKeySet;
pub use crate::auth0::token::Token;

mod cache;
mod config;
mod errors;
mod keyset;
mod token;
mod util;

#[derive(Clone, Debug)]
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

        let jwks: JsonWebKeySet = get_jwks(client_ref, &cache, &config).await?;
        let token: Token = get_token(client_ref, &cache, &config).await?;

        let jwks_lock: Arc<RwLock<JsonWebKeySet>> = Arc::new(RwLock::new(jwks));
        let token_lock: Arc<RwLock<Token>> = Arc::new(RwLock::new(token));

        let _handle: JoinHandle<()> = start(
            jwks_lock.clone(),
            token_lock.clone(),
            client_ref.clone(),
            cache.clone(),
            config,
        )
        .await;

        Ok(Self { token_lock })
    }

    pub fn token(&self) -> Token {
        read(&self.token_lock)
    }
}

async fn start(
    jwks_lock: Arc<RwLock<JsonWebKeySet>>,
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

                let jwks_opt = match JsonWebKeySet::fetch(&client, &config).await {
                    Ok(jwks) => {
                        let _ = cache.put_jwks(&jwks, None).await.log_err("Error caching JWKS");
                        write(&jwks_lock, jwks.clone());
                        Some(jwks)
                    }
                    Err(error) => {
                        tracing::error!("Failed to fetch JWKS. Reason: {:?}", error);
                        None
                    }
                };

                match Token::fetch(&client, &config).await {
                    Ok(token) => {
                        let is_signed: Option<bool> = jwks_opt.map(|j| j.is_signed(&token));
                        tracing::info!("is signed: {}", is_signed.unwrap_or_default());

                        let _ = cache.put_token(&token).await.log_err("Error caching JWT");
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

// Try to fetch the jwks from cache. If it's found return it; fetch from auth0 and put in cache otherwise
async fn get_jwks(
    client_ref: &Client,
    cache_ref: &Arc<dyn Cache>,
    config_ref: &Config,
) -> Result<JsonWebKeySet, Auth0Error> {
    match cache_ref.get_jwks().await? {
        Some(jwks) => Ok(jwks),
        None => {
            let jwks: JsonWebKeySet = JsonWebKeySet::fetch(client_ref, config_ref).await?;
            let _ = cache_ref.put_jwks(&jwks, None).await.log_err("JWKS cache set failed");

            Ok(jwks)
        }
    }
}

// Try to fetch the token from cache. If it's found return it; fetch from auth0 and put in cache otherwise
async fn get_token(client_ref: &Client, cache_ref: &Arc<dyn Cache>, config_ref: &Config) -> Result<Token, Auth0Error> {
    match cache_ref.get_token().await? {
        Some(token) => Ok(token),
        None => {
            let token: Token = Token::fetch(client_ref, config_ref).await?;
            let _ = cache_ref.put_token(&token).await.log_err("JWT cache set failed");

            Ok(token)
        }
    }
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
