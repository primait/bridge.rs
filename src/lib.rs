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

use crate::auth0_config::Auth0Config;
use crate::errors::PrimaBridgeResult;
use crate::token_dispenser::TokenDispenserHandle;

pub mod auth0_config;
pub mod cache;
mod errors;
pub mod prelude;
mod request;
mod response;
mod token_dispenser;

static INTERVAL_CHECK: std::time::Duration = std::time::Duration::from_secs(2);

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
    #[cfg(feature = "auth0")]
    token_dispenser_handle: token_dispenser::TokenDispenserHandle,
}

#[cfg(feature = "blocking")]
impl Bridge {
    #[cfg(not(feature = "auth0"))]
    pub fn new(endpoint: Url) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            endpoint,
        }
    }

    #[cfg(feature = "auth0")]
    pub fn new(endpoint: Url, auth0_config: Auth0Config) -> PrimaBridgeResult<Self> {
        let http_client: reqwest::blocking::Client = reqwest::blocking::Client::new();
        Ok(Self {
            client: http_client.clone(),
            endpoint,
            token_dispenser_handle: Self::new_token_dispenser_handler(http_client, auth0_config)?,
        })
    }

    #[cfg(not(feature = "auth0"))]
    pub fn with_user_agent(endpoint: Url, user_agent: &str) -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .user_agent(user_agent)
                .build()
                .expect("Bridge::with_user_agent()"),
            endpoint,
        }
    }

    #[cfg(feature = "auth0")]
    pub fn with_user_agent(
        endpoint: Url,
        user_agent: &str,
        auth0_config: Auth0Config,
    ) -> PrimaBridgeResult<Self> {
        let http_client: reqwest::blocking::Client = reqwest::blocking::Client::builder()
            .user_agent(user_agent)
            .build()
            .expect("Bridge::with_user_agent()");

        Ok(Self {
            client: http_client.clone(),
            endpoint,
            token_dispenser_handle: Self::new_token_dispenser_handler(http_client, auth0_config)?,
        })
    }

    pub fn get_headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
        let mut headers = HeaderMap::new();
        self.token_dispenser_handle.get_token().map(|t| {
            headers.append(
                HeaderName::from_static("x-token"),
                HeaderValue::from_str(t.as_str()).unwrap(),
            );
        });

        headers
    }

    fn new_token_dispenser_handler(
        http_client: reqwest::blocking::Client,
        auth0_config: Auth0Config,
    ) -> PrimaBridgeResult<TokenDispenserHandle> {
        let token_dispenser_handle = TokenDispenserHandle::run(http_client, auth0_config)?;
        token_dispenser_handle.refresh_token();
        token_dispenser_handle.periodic_check(INTERVAL_CHECK);
        Ok(token_dispenser_handle)
    }
}

#[cfg(not(feature = "blocking"))]
impl Bridge {
    #[cfg(not(feature = "auth0"))]
    pub fn new(endpoint: Url) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint,
            token_dispenser_handle: None,
        }
    }

    #[cfg(feature = "auth0")]
    pub async fn new(endpoint: Url, auth0_config: Auth0Config) -> PrimaBridgeResult<Self> {
        let http_client: reqwest::Client = reqwest::Client::new();

        Ok(Self {
            client: http_client.clone(),
            endpoint,
            token_dispenser_handle: Self::new_token_dispenser_handler(http_client, auth0_config)
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

    #[cfg(feature = "auth0")]
    pub async fn with_user_agent(
        endpoint: Url,
        user_agent: &str,
        auth0_config: Auth0Config,
    ) -> PrimaBridgeResult<Self> {
        let http_client: reqwest::Client = reqwest::Client::builder()
            .user_agent(user_agent)
            .build()
            .expect("Bridge::with_user_agent()");

        Ok(Self {
            client: http_client.clone(),
            endpoint,
            token_dispenser_handle: Self::new_token_dispenser_handler(http_client, auth0_config)
                .await?,
        })
    }

    #[cfg(feature = "auth0")]
    pub async fn get_headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
        let mut headers = HeaderMap::new();
        let token = self.token_dispenser_handle.get_token().await;
        if let Some(t) = token {
            headers.append(
                HeaderName::from_static("x-token"),
                HeaderValue::from_str(t.as_str()).unwrap(),
            );
        }

        headers
    }

    async fn new_token_dispenser_handler(
        http_client: reqwest::Client,
        auth0_config: Auth0Config,
    ) -> PrimaBridgeResult<TokenDispenserHandle> {
        let token_dispenser_handle = TokenDispenserHandle::run(http_client, auth0_config)?;
        token_dispenser_handle.refresh_token().await;
        token_dispenser_handle.periodic_check(INTERVAL_CHECK).await;
        Ok(token_dispenser_handle)
    }
}
