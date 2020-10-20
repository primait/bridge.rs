mod body;
mod request;

use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::{Bridge, Response};
use async_trait::async_trait;
use body::Body;
pub use request::*;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, Url};
use serde::Serialize;
use std::collections::HashMap;
use std::convert::TryInto;
use uuid::Uuid;

pub enum RequestType {
    Rest,
    GraphQL,
}

/// Represent a request that is ready to be delivered to the server
#[async_trait]
pub trait DeliverableRequest<'a>: Sized + 'a {
    /// sets the raw body for the request
    /// it will get delivered in the request as is.
    fn raw_body(
        self,
        body: impl TryInto<Body, Error = PrimaBridgeError>,
    ) -> PrimaBridgeResult<Self>;

    /// sets a serializable body for the request
    fn json_body<B: Serialize>(self, body: B) -> PrimaBridgeResult<Self>;

    /// sets request method. Defaults to GET.
    fn method(self, method: Method) -> Self;

    /// sets the destination path (relative to the url defined in the bridge) for the request
    fn to(self, path: &'a str) -> Self;

    /// ignore the status code, and parse the results even if the response has a wrong status code.
    /// This is useful when you are dealing with an api that return errors with a not 2XX status codes.
    fn ignore_status_code(self) -> Self;

    /// add a custom header to the request
    fn with_custom_headers(self, headers: Vec<(HeaderName, HeaderValue)>) -> Self;

    fn get_id(&self) -> Uuid;
    fn get_bridge(&self) -> &Bridge;
    fn get_path(&self) -> Option<&str>;
    fn endpoint(&self) -> Url;
    fn get_query_pairs(&self) -> &[(&'a str, &'a str)];
    fn get_ignore_status_code(&self) -> bool;
    fn get_method(&self) -> Method;
    fn get_custom_headers(&self) -> &[(HeaderName, HeaderValue)];
    fn get_body(&self) -> Vec<u8>;
    fn get_request_type(&self) -> RequestType;

    #[cfg(feature = "blocking")]
    fn send(&'a self) -> PrimaBridgeResult<Response> {
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
        additional_headers.append(&mut dbg!(self.tracing_headers().to_vec()));
        let request_builder = additional_headers
            .iter()
            .fold(request_builder, |request, (name, value)| {
                request.header(name, value)
            });

        let response = request_builder.body(self.get_body()).send().map_err(|e| {
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

    #[cfg(not(feature = "blocking"))]
    async fn send(&'a self) -> PrimaBridgeResult<Response> {
        use futures_util::future::TryFutureExt;
        use reqwest::header::CONTENT_TYPE;
        let request_id = self.get_id();

        let request_builder = self
            .get_bridge()
            .client
            .request(self.get_method(), self.get_url().as_str())
            .header(CONTENT_TYPE, "application/json")
            .header(
                HeaderName::from_static("x-request-id"),
                &request_id.to_string(),
            );
        let mut additional_headers = vec![];
        additional_headers.append(&mut self.get_custom_headers().to_vec());
        additional_headers.append(&mut self.tracing_headers().to_vec());
        let request_builder = additional_headers
            .iter()
            .fold(request_builder, |request, (name, value)| {
                request.header(name, value)
            });

        let response = request_builder
            .body(self.get_body())
            .send()
            .map_err(|e| PrimaBridgeError::HttpError {
                url: self.get_url(),
                source: e,
            })
            .await?;

        let status_code = response.status();
        if !self.get_ignore_status_code() && !status_code.is_success() {
            return Err(PrimaBridgeError::WrongStatusCode(
                self.get_url(),
                status_code,
            ));
        }

        let response_headers = response.headers().clone();

        let response_body = response
            .text()
            .map_err(|e| PrimaBridgeError::HttpError {
                source: e,
                url: self.get_url(),
            })
            .await?;

        match self.get_request_type() {
            RequestType::Rest => Ok(Response::rest(
                self.get_url(),
                response_body,
                status_code,
                response_headers,
                request_id,
            )),
            RequestType::GraphQL => Ok(Response::graphql(
                self.get_url(),
                response_body,
                status_code,
                response_headers,
                request_id,
            )),
        }
    }

    fn get_url(&self) -> Url {
        let mut final_endpoint = self.endpoint();
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
