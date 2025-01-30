//! Stuff used to provide JWT authentication via Auth0

use reqwest::Client;

pub mod cache;
mod client;
mod config;
mod errors;
mod refresh;
mod token;
mod util;

use cache::Cache;
pub use client::Auth0Client;
pub use config::{CacheType, Config};
pub use errors::Auth0Error;
pub use refresh::RefreshingToken;
pub use token::Token;
pub use util::StalenessCheckPercentage;

#[derive(Clone, Debug)]
pub struct Auth0(RefreshingToken);

impl Auth0 {
    #[deprecated(since = "0.21.0", note = "please use refreshing token")]
    pub async fn new(client: &Client, config: Config) -> Result<Self, Auth0Error> {
        let cache: Box<dyn Cache> = if config.is_inmemory_cache() {
            Box::new(cache::InMemoryCache::new(config.caller.clone(), config.audience.clone()).await?)
        } else {
            Box::new(cache::RedisCache::new(&config).await?)
        };

        let client = client::Auth0Client::from_config(&config, client);
        RefreshingToken::new(
            client,
            config.check_interval,
            config.staleness_check_percentage,
            cache,
            config.audience,
            config.scope,
        )
        .await
        .map(Self)
    }

    pub fn token(&self) -> Token {
        self.0.token().clone()
    }

    pub fn refreshing_token(self) -> RefreshingToken {
        self.0
    }
}
