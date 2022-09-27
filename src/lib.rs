#![cfg_attr(docsrs, feature(doc_cfg))]

//! This crate gives an high level API to execute external requests.
//!
//! It is supposed to give the basics building blocks for building bridges to other services
//! while abstracting the low level stuff like adding custom headers and request tracing.
//!
//! It supports both REST and GraphQL requests.
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
    request::{
        Body, DeliverableRequest, GraphQLMultipart, GraphQLRequest, MultipartFile, MultipartFormFileField, Request,
        RestMultipart, RestRequest,
    },
    response::graphql::{Error, ParsedGraphqlResponse, ParsedGraphqlResponseExt, PossiblyParsedData},
    response::Response,
};

mod builder;
mod errors;
pub mod prelude;
pub mod redirect;
mod request;
mod response;

#[cfg(feature = "auth0")]
pub mod auth0;

/// The bridge instance to issue external requests.
#[derive(Debug)]
pub struct Bridge {
    client: reqwest::Client,
    /// the url this bridge should call to
    endpoint: Url,
    #[cfg(feature = "auth0")]
    auth0_opt: Option<auth0::Auth0>,
}

impl Bridge {
    /// create an instance of [BridgeBuilder]
    pub fn builder() -> BridgeBuilder {
        BridgeBuilder::create()
    }

    #[cfg(feature = "auth0")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth0")))]
    pub fn token(&self) -> Option<crate::auth0::Token> {
        self.auth0_opt.as_ref().map(|auth0| auth0.token())
    }
}
