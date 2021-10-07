use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use reqwest::Client;
use tokio::task::JoinHandle;
use tokio::time::Interval;

pub use config::{Config, StalenessCheckPercentage};
pub use errors::Auth0Error;

use crate::auth0::cache::{Cache, CacheImpl};
use crate::auth0::keyset::JsonWebKeySet;
use crate::auth0::token::Token;

mod cache;
mod config;
mod errors;
mod keyset;
mod token;

#[derive(Clone)]
pub struct Auth0 {
    cache: CacheImpl,
    jwks_lock: Arc<RwLock<JsonWebKeySet>>,
    token_lock: Arc<RwLock<Token>>,
}

impl Auth0 {
    pub async fn new(client_ref: &Client, config: Config) -> Result<Self, Auth0Error> {
        let mut cache: CacheImpl = CacheImpl::new(&config)?;

        let jwks: JsonWebKeySet = get_jwks(client_ref, &mut cache, &config).await?;
        let token: Token = get_token(client_ref, &mut cache, &config).await?;

        let jwks_lock: Arc<RwLock<JsonWebKeySet>> = Arc::new(RwLock::new(jwks));
        let token_lock: Arc<RwLock<Token>> = Arc::new(RwLock::new(token));

        let _: JoinHandle<()> = spawn(
            jwks_lock.clone(),
            token_lock.clone(),
            client_ref.clone(),
            cache.clone(),
            config,
        )
        .await;

        Ok(Self {
            cache,
            jwks_lock,
            token_lock,
        })
    }

    pub fn token(&self) -> Token {
        read(&self.token_lock)
    }
}

async fn spawn(
    jwks_lock: Arc<RwLock<JsonWebKeySet>>,
    token_lock: Arc<RwLock<Token>>,
    client: Client,
    mut cache: CacheImpl,
    config: Config,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker: Interval = tokio::time::interval(config.check_interval().clone());

        loop {
            ticker.tick().await;
            let token: Token = read(&token_lock);

            if token.needs_refresh(&config) {
                match JsonWebKeySet::fetch(&client, &config).await {
                    Ok(jwks) => {
                        let _ = cache.set_jwks(&jwks);
                        write(&jwks_lock, jwks);
                    }
                    Err(error) => println!("{:?}", error), // log here or whatever is not a println
                }

                match Token::fetch(&client, &config).await {
                    Ok(token) => {
                        let _ = cache.set_token(&token);
                        write(&token_lock, token);
                    }
                    Err(error) => println!("{:?}", error), // log here or whatever is not a println
                }
            }
        }
    })
}

// Try to fetch the jwks from cache. If it's found return it; fetch from auth0 and put in cache otherwise
async fn get_jwks(
    client_ref: &Client,
    cache_ref: &mut CacheImpl,
    config_ref: &Config,
) -> Result<JsonWebKeySet, Auth0Error> {
    match cache_ref.get_jwks()? {
        Some(jwks) => Ok(jwks),
        None => {
            let jwks: JsonWebKeySet = JsonWebKeySet::fetch(client_ref, config_ref).await?;
            // NOTE: should we care if this fails? IMO no.
            let _ = cache_ref.set_jwks(&jwks);
            Ok(jwks)
        }
    }
}

// Try to fetch the token from cache. If it's found return it; fetch from auth0 and put in cache otherwise
async fn get_token(
    client_ref: &Client,
    cache_ref: &mut CacheImpl,
    config_ref: &Config,
) -> Result<Token, Auth0Error> {
    match cache_ref.get_token()? {
        Some(token) => Ok(token),
        None => {
            let token: Token = Token::fetch(client_ref, config_ref).await?;
            // NOTE: should we care if this fails? IMO no.
            let _ = cache_ref.set_token(&token);
            Ok(token)
        }
    }
}

fn read<T: Clone>(lock_ref: &Arc<RwLock<T>>) -> T {
    let lock_guard: RwLockReadGuard<T> = lock_ref
        .read()
        .unwrap_or_else(|poison_error| poison_error.into_inner());
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
