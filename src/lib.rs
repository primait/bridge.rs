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
use std::fmt::Debug;

use reqwest::blocking::Client as ReqwestClient;
use reqwest::{StatusCode, Url};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use self::request::{Request, RequestType};

pub mod prelude;
mod request;
mod response;


#[derive(Debug, Error)]
pub enum BridgeRsError {
    #[error("http error while calling {url}, error: {source}")]
    HttpError { url: Url, source: reqwest::Error },
    #[error(transparent)]
    SerializationError(#[from] serde_json::error::Error),
    #[error("selector not found while calling {0}. the data for key `{1}` cannot be found in payload: {2}")]
    SelectorNotFound(Url, String, Value),
    #[error("wrong response status code while calling {0}: {1}")]
    WrongStatusCode(Url, StatusCode),
    #[error("unserializable body. response status code: {status_code}, error: {source}")]
    ResponseBodyNotDeserializable {
        status_code: StatusCode,
        source: serde_json::error::Error,
    },
    #[error("empty body")]
    EmptyBody,
}

pub type BridgeRsResult<T> = Result<T, BridgeRsError>;

/// The bridge instance to issue external requests.
#[derive(Debug)]
pub struct Bridge {
    client: ReqwestClient,
    /// the url this bridge should call to
    endpoint: Url,
}

impl Bridge {
    pub fn new(endpoint: Url) -> Self {
        Self {
            client: ReqwestClient::new(),
            endpoint,
        }
    }

    pub fn request<S: Serialize>(&self, request_type: RequestType<S>) -> Request<S> {
        Request::new(&self, request_type)
    }
}
