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
    builder::BridgeBuilder,
    request::{GraphQLRequest, Request},
    response::graphql::{Error, ParsedGraphqlResponse, PossiblyParsedData},
    response::Response,
};

mod builder;
mod errors;
pub mod prelude;
mod request;
mod response;

/// The bridge instance to issue external requests.
#[derive(Debug)]
pub struct Bridge {
    #[cfg(feature = "blocking")]
    client: reqwest::blocking::Client,
    #[cfg(not(feature = "blocking"))]
    client: reqwest::Client,
    /// the url this bridge should call to
    endpoint: Url,
}

impl Bridge {
    /// create an instance of [BridgeBuilder]
    pub fn builder() -> BridgeBuilder {
        BridgeBuilder::create()
    }

    #[deprecated(since = "0.8.0", note = "please use Bridge::builder().build(...)")]
    pub fn new(endpoint: Url) -> Self {
        Self {
            #[cfg(feature = "blocking")]
            client: reqwest::blocking::Client::new(),
            #[cfg(not(feature = "blocking"))]
            client: reqwest::Client::new(),
            endpoint,
        }
    }

    #[deprecated(since = "0.8.0", note = "please use Bridge::builder().build(...)")]
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
        }
    }
}
