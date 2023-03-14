#![cfg_attr(docsrs, feature(doc_cfg))]

//! This crate gives an high level API to execute external HTTP requests.
//!
//! It is supposed to give the basics building blocks for building bridges to other services
//! while abstracting the low level stuff like adding custom headers and request tracing.
//!
//! It supports both REST and GraphQL requests.
//!
//! You should start by creating a [Bridge] instance.
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
    redirect::RedirectPolicy,
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
mod redirect;
mod request;
mod response;

#[cfg(feature = "auth0")]
#[cfg_attr(docsrs, doc(cfg(feature = "auth0")))]
pub mod auth0;

/// A Bridge instance which can be used to issue HTTP requests to an external service.
#[derive(Debug)]
pub struct Bridge {
    client: reqwest::Client,
    endpoint: Url,
    #[cfg(feature = "auth0")]
    auth0_opt: Option<auth0::Auth0>,
}

impl Bridge {
    /// Creates an instance of a [BridgeBuilder].
    pub fn builder() -> BridgeBuilder {
        BridgeBuilder::create()
    }

    #[cfg(feature = "auth0")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth0")))]
    /// Gets the JWT token used by the Bridge, if it has been configured with Auth0 authentication via
    /// [BridgeBuilder.with_auth0](BridgeBuilder#with_auth0).
    pub fn token(&self) -> Option<auth0::Token> {
        self.auth0_opt.as_ref().map(|auth0| auth0.token())
    }
}
