use std::fmt::Debug;

#[cfg(feature = "blocking")]
use reqwest::blocking::Client as ReqwestClient;
#[cfg(not(feature = "blocking"))]
use reqwest::Client as ReqwestClient;

use reqwest::{
    header::{HeaderName, HeaderValue, CONTENT_TYPE},
    Method, Url,
};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::prelude::*;

/// The Request struct contains all the data to forge a request.
#[derive(Debug)]
pub struct Request<'a, S: Serialize> {
    bridge: &'a Bridge,
    request_type: RequestType<S>,
    custom_headers: Vec<(HeaderName, HeaderValue)>,
    path: Option<&'a str>,
    query_pairs: Vec<(&'a str, &'a str)>,
    ignore_status_code: bool,
}

impl<'a, S: Serialize> Request<'a, S> {
    /// Creates a new request
    pub fn new(bridge: &'a Bridge, request_type: RequestType<S>) -> Self {
        Self {
            bridge,
            request_type,
            custom_headers: vec![],
            path: None,
            query_pairs: vec![],
            ignore_status_code: false,
        }
    }

    /// adds custom headers to a Request
    pub fn with_custom_headers(self, headers: Vec<(HeaderName, HeaderValue)>) -> Self {
        Self {
            custom_headers: headers,
            ..self
        }
    }

    /// routes to request to a subpath of the endpoint defined in the bridge
    pub fn to(self, path: &'a str) -> Self {
        Self {
            path: Some(path),
            ..self
        }
    }

    /// adds a ?name=value to the request endpoint
    pub fn with_query_pair(self, name: &'a str, value: &'a str) -> Self {
        let mut query_pairs = self.query_pairs;
        query_pairs.push((name, value));
        Self {
            query_pairs,
            ..self
        }
    }

    /// try to deserialize body even if the status code is not successful
    pub fn ignore_status_code(self) -> Self {
        Self {
            ignore_status_code: true,
            ..self
        }
    }

    /// issue the external request by consuming the bridge request and
    /// returning a [Response](struct.Response.html)
    #[cfg(feature = "blocking")]
    pub fn send(self) -> PrimaBridgeResult<Response> {
        let request = self.get_request_type();

        let request_builder = self
            .get_client()
            .request(request.get_method(), self.get_url().as_str())
            .header(CONTENT_TYPE, "application/json")
            .header(
                HeaderName::from_static("x-request-id"),
                &request.id().to_string(),
            );

        let mut additional_headers = vec![];
        additional_headers.append(&mut self.custom_headers().to_vec());
        additional_headers.append(&mut self.tracing_headers().to_vec());
        dbg!(&additional_headers);
        let request_builder = additional_headers
            .iter()
            .fold(request_builder, |request, (name, value)| {
                request.header(name, value)
            });

        let response = request_builder
            .body(request.body_as_string()?)
            .send()
            .map_err(|e| PrimaBridgeError::HttpError {
                url: self.get_url(),
                source: e,
            })?;
        let status_code = response.status();
        if !self.ignore_status_code && !status_code.is_success() {
            return Err(PrimaBridgeError::WrongStatusCode(
                self.get_url(),
                status_code,
            ));
        }
        let response_headers = response.headers().clone();

        let response_body = response.text().map_err(|e| PrimaBridgeError::HttpError {
            source: e,
            url: self.get_url(),
        })?;
        let response = match request {
            RequestType::GraphQL(_) => Response::graphql(
                self.get_url(),
                response_body,
                status_code,
                response_headers,
                self.get_request_type().id(),
            ),
            RequestType::Rest(_) => Response::rest(
                self.get_url(),
                response_body,
                status_code,
                response_headers,
                self.get_request_type().id(),
            ),
        };
        Ok(response)
    }

    #[cfg(not(feature = "blocking"))]
    pub async fn send(self) -> PrimaBridgeResult<Response> {
        use futures_util::future::TryFutureExt;
        let request_id = self.get_request_type().id();

        let request_builder = self
            .get_client()
            .request(
                self.get_request_type().get_method(),
                self.get_url().as_str(),
            )
            .header(CONTENT_TYPE, "application/json")
            .header(
                HeaderName::from_static("x-request-id"),
                &request_id.to_string(),
            );
        let mut additional_headers = vec![];
        additional_headers.append(&mut self.custom_headers().to_vec());
        additional_headers.append(&mut self.tracing_headers().to_vec());
        let request_builder = additional_headers
            .iter()
            .fold(request_builder, |request, (name, value)| {
                request.header(name, value)
            });

        let body_as_string = self.get_request_type().body_as_string()?;

        let response = request_builder
            .body(body_as_string)
            .send()
            .map_err(|e| PrimaBridgeError::HttpError {
                url: self.get_url(),
                source: e,
            })
            .await?;

        let status_code = response.status();
        if !self.ignore_status_code && !status_code.is_success() {
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

        if self.get_request_type().is_graphql() {
            Ok(Response::graphql(
                self.get_url(),
                response_body,
                status_code,
                response_headers,
                request_id,
            ))
        } else {
            Ok(Response::rest(
                self.get_url(),
                response_body,
                status_code,
                response_headers,
                request_id,
            ))
        }
    }

    fn get_client(&self) -> &ReqwestClient {
        &self.bridge.client
    }

    fn get_request_type(&self) -> &RequestType<S> {
        &self.request_type
    }

    fn custom_headers(&self) -> &[(HeaderName, HeaderValue)] {
        &self.custom_headers
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

    fn get_path(&self) -> Option<&str> {
        self.path
    }

    fn get_url(&self) -> Url {
        let mut endpoint = self.bridge.endpoint.clone();
        let path_segments = self.bridge.endpoint.path_segments();
        let path = self.get_path();

        endpoint = match path {
            Some(path) => {
                let mut parts: Vec<&str> = path_segments
                    .map_or_else(Vec::new, |ps| ps.collect())
                    .into_iter()
                    .filter(|p| p != &"")
                    .collect();
                parts.push(path);
                endpoint.set_path(&parts.join("/"));
                endpoint
            }
            _ => endpoint,
        };

        self.query_pairs
            .iter()
            .fold(endpoint, |mut url, (name, value)| {
                url.query_pairs_mut().append_pair(name, value);
                url
            })
    }
}

#[derive(Debug)]
pub enum RequestType<S: Serialize> {
    GraphQL(GraphQL<S>),
    Rest(Rest<S>),
}

#[derive(Debug)]
pub struct GraphQL<S: Serialize> {
    request_id: Uuid,
    body: GraphQLBody<S>,
}

#[derive(Debug, Serialize)]
pub struct GraphQLBody<T> {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<T>,
}

#[derive(Debug)]
pub struct Rest<S: Serialize> {
    request_id: Uuid,
    body: Option<RestBody<S>>,
    method: Method,
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct RestBody<S: Serialize> {
    value: S,
}

impl<S: Serialize> RequestType<S> {
    pub fn id(&self) -> Uuid {
        match self {
            RequestType::GraphQL(request) => request.request_id,
            RequestType::Rest(request) => request.request_id,
        }
    }

    pub fn body_as_string(&self) -> PrimaBridgeResult<String> {
        match self {
            RequestType::GraphQL(request) => Ok(serde_json::to_string(&request.body)?),
            RequestType::Rest(request) => Ok(serde_json::to_string(&request.body)?),
        }
    }

    pub fn is_graphql(&self) -> bool {
        match self {
            RequestType::GraphQL(_) => true,
            RequestType::Rest(_) => false,
        }
    }

    pub fn is_rest(&self) -> bool {
        match self {
            RequestType::GraphQL(_) => false,
            RequestType::Rest(_) => true,
        }
    }

    pub fn get_method(&self) -> Method {
        match self {
            RequestType::GraphQL(_) => Method::POST,
            RequestType::Rest(request) => request.method.clone(),
        }
    }

    pub fn rest(body: Option<S>, method: Method) -> Self {
        let request_id = Uuid::new_v4();
        match body {
            None => Self::Rest(Rest {
                request_id,
                method,
                body: None,
            }),
            Some(body) => Self::Rest(Rest {
                request_id,
                body: Some(RestBody { value: body }),
                method,
            }),
        }
    }

    pub fn graphql(query: &str, variables: Option<S>) -> Self {
        Self::GraphQL(GraphQL {
            request_id: Uuid::new_v4(),
            body: GraphQLBody {
                query: query.to_string(),
                variables,
            },
        })
    }
}
