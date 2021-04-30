//! This crate gives an high level api to execute external request.
//!
//! It is supposed to give the basics building blocks for building bridges to the external world
//! while abstracting the low level stuffs like adding custom headers and request tracing.
//!
//! Right now it supports Rest and GraphQL requests.
//!
//! You should start by creating a [Bridge](struct.Bridge.html) instance.
//! This instance should live for all the application lifetime.
//!
//! **Do not create a new bridge on every request!**
//!
//! You should use something like [once_cell](https://crates.io/crates/once_cell) or [lazy_static](https://crates.io/crates/lazy_static), or some sort of inversion of control container to
//! pass around.
//!
//! The bridge implement a type state pattern to build the external request.

use reqwest::Url;

pub use self::{
    request::{GraphQLRequest, Request},
    response::graphql::{Error, ParsedGraphqlResponse, PossiblyParsedData},
    response::Response,
};

#[cfg(all(not(feature = "blocking"), feature = "auth0"))]
pub mod auth0_config;
#[cfg(all(not(feature = "blocking"), feature = "auth0"))]
pub mod cache;
#[cfg(all(not(feature = "blocking"), feature = "auth0"))]
mod token_dispenser;

#[cfg(all(not(feature = "blocking"), feature = "auth0"))]
use crate::cache::{Cache, CacheImpl};

mod errors;
pub mod prelude;
mod request;
mod response;

#[cfg(all(feature = "auth0", not(feature = "blocking")))]
static INTERVAL_CHECK: std::time::Duration = std::time::Duration::from_secs(1);
#[cfg(all(feature = "auth0", not(feature = "blocking")))]
static INTERVAL_JWKS_CHECK: std::time::Duration = std::time::Duration::from_secs(60);

/// The bridge instance to issue external requests.
#[derive(Debug)]
pub struct Bridge {
    #[cfg(feature = "blocking")]
    client: reqwest::blocking::Client,
    #[cfg(not(feature = "blocking"))]
    client: reqwest::Client,
    /// the url this bridge should call to
    endpoint: Url,
    /// the auth0 token process. Covertly refresh token and expose get token api.
    #[cfg(all(feature = "auth0", not(feature = "blocking")))]
    token_dispenser_handle: token_dispenser::TokenDispenserHandle,
}

#[cfg(feature = "blocking")]
impl Bridge {
    pub fn new(endpoint: Url) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            endpoint,
        }
    }

    pub fn with_user_agent(endpoint: Url, user_agent: &str) -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .user_agent(user_agent)
                .build()
                .expect("Bridge::with_user_agent()"),
            endpoint,
        }
    }

    pub fn get_headers(&self) -> reqwest::header::HeaderMap {
        reqwest::header::HeaderMap::new()
    }
}

#[cfg(not(feature = "blocking"))]
impl Bridge {
    #[cfg(not(feature = "auth0"))]
    pub fn new(endpoint: Url) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint,
        }
    }

    // todo: can we think about add http client and cache as ref here? And create new version where
    // we create http client and cache
    #[cfg(feature = "auth0")]
    pub async fn new(
        endpoint: Url,
        auth0_config: auth0_config::Auth0Config,
    ) -> errors::PrimaBridgeResult<Self> {
        let http_client: reqwest::Client = reqwest::Client::new();
        let cache = CacheImpl::new(&auth0_config)?;

        Ok(Self {
            client: http_client.clone(),
            endpoint,
            token_dispenser_handle: Self::new_token_dispenser_handler(
                &http_client,
                &cache,
                &auth0_config,
            )
            .await?,
        })
    }

    #[cfg(not(feature = "auth0"))]
    pub fn with_user_agent(endpoint: Url, user_agent: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(user_agent)
                .build()
                .expect("Bridge::with_user_agent()"),
            endpoint,
        }
    }

    // todo: think about this function. I would leave the choice of how to build the http client to the caller
    // avoiding the proliferation of http clients
    #[cfg(feature = "auth0")]
    pub async fn with_user_agent(
        endpoint: Url,
        user_agent: &str,
        auth0_config: auth0_config::Auth0Config,
    ) -> errors::PrimaBridgeResult<Self> {
        let http_client: reqwest::Client = reqwest::Client::builder()
            .user_agent(user_agent)
            .build()
            .expect("Bridge::with_user_agent()");

        let cache = CacheImpl::new(&auth0_config)?;

        Ok(Self {
            token_dispenser_handle: Self::new_token_dispenser_handler(
                &http_client,
                &cache,
                &auth0_config,
            )
            .await?,
            client: http_client,
            endpoint,
        })
    }

    #[cfg(feature = "auth0")]
    pub async fn get_headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderValue};
        let mut headers = HeaderMap::new();
        self.token_dispenser_handle.get_token().await.map(|t| {
            headers.append(
                reqwest::header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", t)).unwrap(),
            );
        });
        headers
    }

    #[cfg(not(feature = "auth0"))]
    pub async fn get_headers(&self) -> reqwest::header::HeaderMap {
        reqwest::header::HeaderMap::new()
    }

    #[cfg(feature = "auth0")]
    async fn new_token_dispenser_handler(
        http_client: &reqwest::Client,
        cache: &CacheImpl,
        auth0_config: &auth0_config::Auth0Config,
    ) -> errors::PrimaBridgeResult<token_dispenser::TokenDispenserHandle> {
        let token_dispenser_handle: token_dispenser::TokenDispenserHandle =
            token_dispenser::TokenDispenserHandle::run(http_client, cache, auth0_config)?;
        token_dispenser_handle.fetch_jwks().await;
        token_dispenser_handle
            .periodic_jwks_check(INTERVAL_JWKS_CHECK)
            .await;
        token_dispenser_handle.refresh_token().await;
        token_dispenser_handle.periodic_check(INTERVAL_CHECK).await;
        Ok(token_dispenser_handle)
    }
}
