use crate::prelude::Bridge;
mod request;

use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::Response;
pub use request::*;
use reqwest::header::HeaderName;
use reqwest::{Method, StatusCode, Url};
use std::convert::{TryFrom, TryInto};

#[derive(Clone)]
pub struct Body {
    inner: Vec<u8>,
}

impl TryFrom<Body> for Vec<u8> {
    type Error = PrimaBridgeError;

    fn try_from(value: Body) -> Result<Self, Self::Error> {
        Ok(value.inner)
    }
}

impl Default for Body {
    fn default() -> Self {
        Self { inner: vec![] }
    }
}

impl From<Body> for reqwest::blocking::Body {
    fn from(body: Body) -> Self {
        body.into()
    }
}

impl TryFrom<String> for Body {
    type Error = PrimaBridgeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: value.into_bytes(),
        })
    }
}

impl TryFrom<Option<String>> for Body {
    type Error = PrimaBridgeError;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Body::default()),
            Some(val) => Ok(Self {
                inner: val.into_bytes(),
            }),
        }
    }
}

impl TryFrom<Vec<u8>> for Body {
    type Error = PrimaBridgeError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self { inner: value })
    }
}

trait Request {
    fn rest(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge)
    }
    fn graphql(bridge: &Bridge) -> GraphQLRequest {
        GraphQLRequest::new(bridge)
    }
}

pub trait SendableRequest: Sized {
    fn send(&self) -> PrimaBridgeResult<Response>;
    fn get_url(&self) -> Url;
    fn body(self, body: impl TryInto<Body, Error = PrimaBridgeError>) -> PrimaBridgeResult<Self>;
    fn method(self, method: Method) -> Self;
}

impl SendableRequest for RestRequest<'_> {
    fn send(&self) -> PrimaBridgeResult<Response> {
        let request_builder = self
            .get_client()
            .request(self.get_method(), self.get_url().as_str())
            .header(
                HeaderName::from_static("x-request-id"),
                &self.id.to_string(),
            );
        let response = request_builder.body(self.get_body()?).send().map_err(|e| {
            PrimaBridgeError::HttpError {
                url: self.get_url(),
                source: e,
            }
        })?;
        let status_code = response.status();
        let response_headers = response.headers().clone();
        let response_body = response.text().map_err(|e| PrimaBridgeError::HttpError {
            source: e,
            url: self.get_url(),
        })?;

        Ok(Response::rest(
            Self::get_url(self),
            response_body,
            status_code,
            response_headers,
            self.id,
        ))
    }

    fn get_url(&self) -> Url {
        self.bridge.endpoint.clone()
    }

    fn body(self, body: impl TryInto<Body, Error = PrimaBridgeError>) -> PrimaBridgeResult<Self> {
        self.body(body)
    }

    fn method(self, method: Method) -> Self {
        self.method(method)
    }
}
