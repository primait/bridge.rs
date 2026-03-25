//! Errors

use std::convert::Infallible;
use std::fmt::{self, Debug, Display};
use std::str::Utf8Error;

use reqwest::{StatusCode, Url};
use serde_json::Value;
use thiserror::Error;

pub type PrimaBridgeResult<T> = Result<T, PrimaBridgeError>;

#[derive(Debug, Error)]
pub enum PrimaBridgeError {
    #[error("http error while calling {url}, error: {source}")]
    HttpError { url: Url, source: reqwest::Error },
    #[error(transparent)]
    SerializationError(#[from] serde_json::error::Error),
    #[error(
        "selector not found while calling {url}. the data for key `{key}` cannot be found in payload: {payload}",
        url = .0.0, key = .0.1, payload = .0.2
    )]
    SelectorNotFound(Box<(Url, String, Value)>),
    #[error("wrong response status code while calling {0}: {1}")]
    WrongStatusCode(Url, StatusCode),
    #[error("unserializable body. response status code: {status_code}, error: {source}")]
    ResponseBodyNotDeserializable {
        status_code: StatusCode,
        source: serde_json::error::Error,
    },
    #[error("failed to deserialize JSON response at path {path}: {error}", path = .error.path())]
    DeserializationError {
        body_structure: BodyStructure,
        #[source]
        error: serde_path_to_error::Error<serde_json::Error>,
    },
    #[error("empty body")]
    EmptyBody,
    #[error("variables map is malformed or provided path not step into objects only")]
    MalformedVariables,
    #[error("multipart file has invalid mime type: {0}")]
    InvalidMultipartFileMimeType(String),
    #[error("the response body id not valid utf-8. error: {source}")]
    Utf8Error { source: Utf8Error },
    #[error("some error occurred in a middleware layer")]
    MiddlewareError(reqwest_middleware::Error),
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

/// Holds a JSON value navigated to the error path, and renders its structural
/// shape (without actual values) in its `Display`/`Debug` impl.
pub struct BodyStructure(Value);

impl BodyStructure {
    pub fn at_path(value: &Value, path: &serde_path_to_error::Path) -> Self {
        let mut current = value;

        for segment in path.iter() {
            match segment {
                serde_path_to_error::Segment::Map { key } => {
                    let Some(inner) = current.get(key.as_ref() as &str) else {
                        break;
                    };
                    current = inner;
                }
                serde_path_to_error::Segment::Seq { index } => {
                    let Some(inner) = current.get(*index) else {
                        break;
                    };
                    current = inner;
                }
                _ => break,
            }
        }

        Self(current.clone())
    }
}

impl Display for BodyStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", JsonShape(&self.0))
    }
}

impl Debug for BodyStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

struct JsonShape<'a>(&'a Value);

impl Display for JsonShape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Value::Null => write!(f, "null"),
            Value::Bool(_) => write!(f, "bool"),
            Value::Number(_) => write!(f, "number"),
            Value::String(_) => write!(f, "string"),
            Value::Array(arr) => {
                let mut list = f.debug_list();
                if let Some(first) = arr.first() {
                    list.entry(&format_args!("{}", JsonShape(first)));
                    if arr.len() > 1 {
                        list.entry(&format_args!("..{} total", arr.len()));
                    }
                }
                list.finish()
            }
            Value::Object(map) => {
                let mut debug_map = f.debug_map();
                for (key, value) in map.iter() {
                    debug_map.entry(&format_args!("{key}"), &format_args!("{}", JsonShape(value)));
                }
                debug_map.finish()
            }
        }
    }
}
