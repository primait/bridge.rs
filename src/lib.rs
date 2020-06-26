//! # prima_bridge
//!
//! This module is responsible of issuing external request.
//! It is supposed to give the basics building blocks for building bridges to the external world
//! while abstracting the low level stuffs like adding our custom headers and request tracing.
//!
//! Right now it supports Rest and GraphQL requests.
//!
//! You should start by creating a [Bridge](struct.Bridge.html) instance.
//! This instance should live for all the application lifetime.
//!
//! **Do not create a new bridge on every request!**
//!
//! You should use something like lazy_static, or some sort of inversion of control container to
//! pass around.
//!
//! The bridge implement a type state pattern to build the external request. Follow the types!

mod errors;
pub mod prelude;
mod request;
mod response;

use self::request::{Request, RequestType};
use reqwest::Url;
use serde::Serialize;

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
    #[cfg(feature = "blocking")]
    pub fn new(endpoint: Url) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            endpoint,
        }
    }

    #[cfg(not(feature = "blocking"))]
    pub fn new(endpoint: Url) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint,
        }
    }

    pub fn request<S: Serialize>(&self, request_type: RequestType<S>) -> Request<S> {
        Request::new(&self, request_type)
    }
}
