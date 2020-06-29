use reqwest::{StatusCode, Url};
use serde_json::Value;
use thiserror::Error;

pub type BridgeRsResult<T> = Result<T, BridgeRsError>;

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
