use std::cell::RefCell;
use std::future::Future;
use std::sync::RwLock;
use std::time::Duration;

use chrono::Utc;
use reqwest::Client;
use tokio::time::Interval;

use cache::Redis;
use config::Config;
use keyset::JsonWebKeySet;

use crate::auth0::token::Token;
use crate::config::StalenessCheckPercentage;
use crate::errors::{PrimaBridgeError, PrimaBridgeResult};

pub mod cache;
pub mod config;
mod keyset;
mod token;

#[derive(Clone)]
pub struct Authentication {
    client: Client,
    cache: Redis,
    config: Config,
    token_ref: RwLock<Token>,
}

impl Authentication {
    pub async fn new(client: &Client, config: Config) -> PrimaBridgeResult<Self> {
        let mut redis: Redis = Redis::new(&config)?;
        let client: Client = client.clone();

        // If token do not exist in Redis fetch the token from authority and store it in Redis.
        let token: Token = match redis.get_token()? {
            None => fetch_token(&client, &mut redis, &config).await?,
            Some(token) if token.needs_refresh(&config) => {
                fetch_token(&client, &mut redis, &config).await?
            }
            Some(token) => token,
        };

        // Here get Json Web Token and Json Web Key Set in order to be sure that the app will start
        // with all needed information.
        let token: Token = refresh(&client, &mut redis, &config).await?;

        // Spawn a thread that keep Json Web Token and Json Web Key Set up to date
        let thread_client: Client = client.clone();
        let mut thread_redis: Redis = redis.clone();
        let thread_config: Config = config.clone();

        let me: Self = Authentication {
            client: client.clone(),
            cache: redis,
            config,
            token_ref: RwLock::new(token),
        };

        let thread_me: Authentication = me.clone();
        tokio::spawn(async move {
            let mut ticker: Interval = tokio::time::interval(Duration::from_secs(60));
            // First tick completes immediately: https://docs.rs/tokio/1.12.0/tokio/time/fn.interval.html
            ticker.tick().await;
            loop {
                ticker.tick().await;
                match refresh(&thread_client, &mut thread_redis, &thread_config).await {
                    Ok(_) => (),
                    Err(err) => println!("Error while fetching jwks {:?}", err), //error!("Error while fetching jwks {:?}", err)
                }
            }
        });

        Ok(me)
    }

    pub fn token(&self) -> Token {
        self.token_ref.clone().take()
    }
}

// Refresh JWT and JWKS data
async fn refresh(client: &Client, redis: &mut Redis, config: &Config) -> PrimaBridgeResult<Token> {
    let _ = fetch_jwks(client, redis, config).await?;
    Ok(fetch_token(client, redis, config).await?)
}

async fn fetch_token(
    client: &Client,
    redis: &mut Redis,
    config: &Config,
) -> PrimaBridgeResult<Token> {
    let token: Token = Token::fetch(client, config).await?;
    redis.set_token(&token);
    Ok(token)
}

async fn fetch_jwks(
    client: &Client,
    redis: &mut Redis,
    config: &Config,
) -> PrimaBridgeResult<JsonWebKeySet> {
    let jwks: JsonWebKeySet = JsonWebKeySet::fetch(client, config).await?;
    redis.set_jwks(&jwks)?;
    Ok(jwks)
}
