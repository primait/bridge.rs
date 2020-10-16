mod request;

use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::{Bridge, Response};
pub use request::*;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, StatusCode, Url};
use serde::Serialize;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use uuid::Uuid;

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

enum RequestType {
    Rest,
    GraphQL,
}

pub trait DeliverableRequest<'a>: Sized + 'a {
    fn raw_body(
        self,
        body: impl TryInto<Body, Error = PrimaBridgeError>,
    ) -> PrimaBridgeResult<Self>;
    fn json_body<B: Serialize>(self, body: B) -> PrimaBridgeResult<Self>;
    fn method(self, method: Method) -> Self;
    fn to(self, path: &'a str) -> Self;
    fn ignore_status_code(self) -> Self;
    fn with_custom_headers(self, headers: Vec<(HeaderName, HeaderValue)>) -> Self;

    fn get_id(&self) -> Uuid;
    fn get_bridge(&self) -> &Bridge;
    fn get_path(&self) -> Option<&str>;
    fn endpoint(&self) -> Url;
    fn get_query_pairs(&self) -> &[(&'a str, &'a str)];
    fn get_ignore_status_code(&self) -> bool;
    fn get_method(&self) -> Method;
    fn get_custom_headers(&self) -> &[(HeaderName, HeaderValue)];
    fn get_body(&self) -> PrimaBridgeResult<Vec<u8>>;
    fn get_request_type(&self) -> RequestType;

    fn send(&self) -> PrimaBridgeResult<Response> {
        let request_builder = self
            .get_bridge()
            .client
            .request(self.get_method(), self.get_url().as_str())
            .header(
                HeaderName::from_static("x-request-id"),
                &self.get_id().to_string(),
            );

        let mut additional_headers = vec![];
        additional_headers.append(&mut self.get_custom_headers().to_vec());
        additional_headers.append(&mut self.tracing_headers().to_vec());
        let request_builder = additional_headers
            .iter()
            .fold(request_builder, |request, (name, value)| {
                request.header(name, value)
            });

        let response = request_builder.body(self.get_body()?).send().map_err(|e| {
            PrimaBridgeError::HttpError {
                url: self.get_url(),
                source: e,
            }
        })?;
        let status_code = response.status();
        if !self.get_ignore_status_code() && !status_code.is_success() {
            return Err(PrimaBridgeError::WrongStatusCode(
                Self::get_url(self),
                status_code,
            ));
        }
        let response_headers = response.headers().clone();
        let response_body = response.text().map_err(|e| PrimaBridgeError::HttpError {
            source: e,
            url: self.get_url(),
        })?;

        match self.get_request_type() {
            RequestType::Rest => Ok(Response::rest(
                Self::get_url(self),
                response_body,
                status_code,
                response_headers,
                self.get_id(),
            )),
            RequestType::GraphQL => Ok(Response::graphql(
                Self::get_url(self),
                response_body,
                status_code,
                response_headers,
                self.get_id(),
            )),
        }
    }

    fn get_url(&self) -> Url {
        let mut final_endpoint = self.endpoint();
        let path_segments = self.endpoint().path_segments();
        let path = self.get_path();
        let endpoint = match path {
            Some(path) => {
                let ep = self.endpoint();

                let mut parts: Vec<&str> = ep
                    .path_segments()
                    .map_or_else(Vec::new, |ps| ps.collect())
                    .into_iter()
                    .filter(|p| p != &"")
                    .collect();
                parts.push(path);
                final_endpoint.set_path(&parts.join("/"));
                final_endpoint
            }
            _ => final_endpoint,
        };

        self.get_query_pairs()
            .iter()
            .fold(endpoint, |mut url, (name, value)| {
                url.query_pairs_mut().append_pair(name, value);
                url
            })
    }

    #[cfg(feature = "tracing_opentelemetry")]
    fn tracing_headers(&self) -> Vec<(HeaderName, HeaderValue)> {
        use opentelemetry::api::HttpTextFormat;
        use tracing_opentelemetry::OpenTelemetrySpanExt;

        let context = tracing::Span::current().context();
        let mut tracing_headers: HashMap<String, String> = HashMap::new();
        let extractor = opentelemetry::api::TraceContextPropagator::new();
        extractor.inject_context(&context, &mut tracing_headers);

        tracing_headers
            .iter()
            .flat_map(|(name, value)| {
                let header_name = HeaderName::from_bytes(name.as_bytes());
                let header_value = HeaderValue::from_bytes(value.as_bytes());
                match (header_name, header_value) {
                    (Ok(valid_header_name), Ok(valid_header_value)) => {
                        vec![(valid_header_name, valid_header_value)]
                    }
                    _ => vec![],
                }
            })
            .collect()
    }

    #[cfg(not(feature = "tracing_opentelemetry"))]
    fn tracing_headers(&self) -> Vec<(HeaderName, HeaderValue)> {
        vec![]
    }
}
