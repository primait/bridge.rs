use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Method, Url};
use serde::Serialize;
use uuid::Uuid;

pub use body::{Body, GraphQLBody};
pub use request_type::{GraphQLRequest, Request, RestRequest};

use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::{Bridge, Response};
use recloser::Error;

mod body;
mod request_type;

pub enum RequestType {
    Rest,
    #[allow(clippy::upper_case_acronyms)]
    GraphQL,
}

/// Represent a request that is ready to be delivered to the server
#[async_trait]
pub trait DeliverableRequest<'a>: Sized + 'a {
    /// sets the raw body for the request
    /// it will get delivered in the request as is.
    fn raw_body(self, body: impl Into<Body>) -> Self;

    /// sets a serializable body for the request
    fn json_body<B: Serialize>(self, body: &B) -> PrimaBridgeResult<Self>;

    /// sets request method. Defaults to GET.
    fn method(self, method: Method) -> Self;

    /// sets the destination path (relative to the url defined in the bridge) for the request
    fn to(self, path: &'a str) -> Self;

    /// ignore the status code, and parse the results even if the response has a wrong status code.
    /// This is useful when you are dealing with an api that return errors with a not 2XX status codes.
    fn ignore_status_code(self) -> Self;

    /// set request timeout
    fn set_timeout(self, timeout: Duration) -> Self;

    /// get request timeout
    fn get_timeout(&self) -> Duration;

    /// adds a new set of headers to the request. Any header already present gets merged.
    fn set_custom_headers(self, headers: HeaderMap) -> Self;

    /// adds a new set of headers to the request. Any header already present gets merged.
    fn add_custom_headers(self, headers: Vec<(HeaderName, HeaderValue)>) -> Self {
        let mut custom_headers = HeaderMap::new();
        custom_headers.extend(self.get_custom_headers());
        custom_headers.extend(headers);
        self.set_custom_headers(custom_headers)
    }

    /// adds a new set of headers to the request. Any header already present gets removed.
    fn set_query_pairs(self, query_pairs: Vec<(&'a str, &'a str)>) -> Self;

    /// add a custom header to the set of request headers
    fn with_custom_headers(self, headers: Vec<(HeaderName, HeaderValue)>) -> Self {
        self.add_custom_headers(headers)
    }

    /// add a custom query string param
    fn with_query_pair(self, name: &'a str, value: &'a str) -> Self {
        let mut query_pairs = self.get_query_pairs().to_vec();
        query_pairs.push((name, value));
        self.set_query_pairs(query_pairs)
    }

    /// add a custom query string param
    fn with_query_pairs(self, pairs: Vec<(&'a str, &'a str)>) -> Self {
        let query_pairs = self.get_query_pairs().to_vec();
        let query_pairs = pairs.iter().fold(query_pairs, |mut acc, (name, value)| {
            acc.push((name, value));
            acc
        });

        self.set_query_pairs(query_pairs)
    }

    /// retrurns a unique id for the request
    fn get_id(&self) -> Uuid;

    #[doc(hidden)]
    fn get_bridge(&self) -> &Bridge;

    #[doc(hidden)]
    fn get_path(&self) -> Option<&str>;

    #[doc(hidden)]
    fn endpoint(&self) -> Url;

    #[doc(hidden)]
    fn get_query_pairs(&self) -> &[(&'a str, &'a str)];

    #[doc(hidden)]
    fn get_ignore_status_code(&self) -> bool;

    #[doc(hidden)]
    fn get_method(&self) -> Method;

    #[doc(hidden)]
    fn get_custom_headers(&self) -> HeaderMap;

    #[cfg(feature = "auth0")]
    #[doc(hidden)]
    fn get_auth0(&self) -> &Option<crate::auth0::Auth0>;

    #[cfg(feature = "auth0")]
    #[doc(hidden)]
    fn get_auth0_headers(&self) -> HeaderMap {
        match self.get_auth0().as_ref().map(|auth0| auth0.token()) {
            None => HeaderMap::new(),
            Some(token) => {
                let mut header_map: HeaderMap = HeaderMap::new();
                let header_value: HeaderValue = HeaderValue::from_str(token.to_bearer().as_str())
                    // This shouldn't happen. Token must be writeable and shouldn't contain invalid characters (eg. \n)
                    .expect("Failed to create bearer header");

                header_map.append(reqwest::header::AUTHORIZATION, header_value);
                header_map
            }
        }
    }

    fn get_all_headers(&self) -> HeaderMap {
        let mut additional_headers = HeaderMap::new();
        additional_headers.extend(self.get_custom_headers());
        additional_headers.extend(self.tracing_headers());
        #[cfg(feature = "auth0")]
        additional_headers.extend(self.get_auth0_headers());
        additional_headers
    }

    #[doc(hidden)]
    fn get_body(&self) -> Vec<u8>;

    #[doc(hidden)]
    fn get_request_type(&self) -> RequestType;

    async fn do_send<T>(
        bridge: &Bridge,
        request_builder: reqwest::RequestBuilder,
        url: &Url,
        body: T,
    ) -> PrimaBridgeResult<reqwest::Response>
    where
        T: Into<reqwest::Body> + Send,
    {
        match &bridge.circuit_breaker {
            None => {
                request_builder
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| PrimaBridgeError::HttpError {
                        url: url.clone(),
                        source: e,
                    })
            }
            Some(circuit_breaker) => circuit_breaker
                .call(request_builder.body(body).send())
                .await
                .map_err(|e| match e {
                    Error::Inner(inner) => PrimaBridgeError::HttpError {
                        url: url.clone(),
                        source: inner,
                    },
                    Error::Rejected => PrimaBridgeError::CircuitBreakerOpen,
                }),
        }
    }

    async fn send(&'a self) -> PrimaBridgeResult<Response> {
        use futures_util::TryFutureExt;

        let request_id = self.get_id();
        let url = self.get_url();

        let request_builder = self
            .get_bridge()
            .client
            .request(self.get_method(), url.as_str())
            .timeout(self.get_timeout())
            .header(
                HeaderName::from_static("x-request-id"),
                &request_id.to_string(),
            )
            .headers(self.get_all_headers());

        let response =
            Self::do_send(self.get_bridge(), request_builder, &url, self.get_body()).await?;

        let status_code = response.status();
        if !self.get_ignore_status_code() && !status_code.is_success() {
            return Err(PrimaBridgeError::WrongStatusCode(url.clone(), status_code));
        }

        let response_headers = response.headers().clone();

        let response_body = response
            .bytes()
            .map_err(|e| PrimaBridgeError::HttpError {
                source: e,
                url: url.clone(),
            })
            .await?
            .to_vec();

        match self.get_request_type() {
            RequestType::Rest => Ok(Response::rest(
                url.clone(),
                response_body,
                status_code,
                response_headers,
                request_id,
            )),
            RequestType::GraphQL => Ok(Response::graphql(
                url.clone(),
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
    fn tracing_headers(&self) -> HeaderMap {
        use opentelemetry::propagation::text_map_propagator::TextMapPropagator;
        use tracing_opentelemetry::OpenTelemetrySpanExt;

        let context = tracing::Span::current().context();
        let mut tracing_headers: HashMap<String, String> = HashMap::new();
        let extractor = opentelemetry::sdk::propagation::TraceContextPropagator::new();
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
