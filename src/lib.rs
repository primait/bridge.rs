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
use crate::token_dispenser::{TokenDispenser, TokenDispenserHandle};
use tokio::sync::mpsc;

mod errors;
pub mod prelude;
mod request;
mod response;
mod token_dispenser;

/// The bridge instance to issue external requests.
#[derive(Debug)]
pub struct Bridge {
    #[cfg(feature = "blocking")]
    client: reqwest::blocking::Client,
    #[cfg(not(feature = "blocking"))]
    client: reqwest::Client,
    /// the url this bridge should call to
    endpoint: Url,
    token_dispenser_handle: Option<TokenDispenserHandle>,
}

impl Bridge {
    pub fn new(endpoint: Url) -> Self {
        Self {
            #[cfg(feature = "blocking")]
            client: reqwest::blocking::Client::new(),
            #[cfg(not(feature = "blocking"))]
            client: reqwest::Client::new(),
            endpoint,
            token_dispenser_handle: None,
        }
    }

    pub fn with_user_agent(endpoint: Url, user_agent: &str) -> Self {
        Self {
            #[cfg(feature = "blocking")]
            client: reqwest::blocking::Client::builder()
                .user_agent(user_agent)
                .build()
                .expect("Bridge::with_user_agent()"),
            #[cfg(not(feature = "blocking"))]
            client: reqwest::Client::builder()
                .user_agent(user_agent)
                .build()
                .expect("Bridge::with_user_agent()"),
            endpoint,
            token_dispenser_handle: None,
        }
    }

    pub async fn with_auth0_authentication(&mut self, endpoint: Url, audience: &str) {
        let mut token_dispenser_handle = TokenDispenserHandle::run(endpoint, audience);
        let new_token = token_dispenser_handle.refresh_token().await;
        self.token_dispenser_handle = Some(token_dispenser_handle);
    }
}
