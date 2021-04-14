use std::convert::Infallible;

use reqwest::{StatusCode, Url};
use serde_json::Value;
use std::fmt::Debug;
use std::str::Utf8Error;
use thiserror::Error;

pub type PrimaBridgeResult<T> = Result<T, PrimaBridgeError>;

#[derive(Debug, Error)]
pub enum PrimaBridgeError {
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
    #[error("http error while fetching auth0 token, error: {0}")]
    Auth0TokenFetchError(reqwest::Error),
    #[error("http error while deserializing auth0 token, error: {0}")]
    Auth0TokenDeserializeError(reqwest::Error),
    #[error(transparent)]
    Auth0CacheError(#[from] redis::RedisError),
    #[error("the response body id not valid utf-8. error: {source}")]
    Utf8Error { source: Utf8Error },
}

impl PrimaBridgeError {
    pub fn utf8_error(source: Utf8Error) -> Self {
        Self::Utf8Error { source }
    }
}

impl From<Infallible> for PrimaBridgeError {
    fn from(_: Infallible) -> Self {
        unimplemented!()
    }
}
