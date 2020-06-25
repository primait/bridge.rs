use std::fmt::Debug;

use reqwest::blocking::Client as ReqwestClient;
use reqwest::{
    header::{HeaderName, HeaderValue, CONTENT_TYPE},
    Method, Url,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::errors::{BridgeRsError, BridgeRsResult};
use crate::prelude::*;

pub struct Request<'a, S: Serialize> {
    bridge: &'a Bridge,
    request_type: RequestType<S>,
    custom_headers: Vec<(HeaderName, HeaderValue)>,
    path: Option<&'a str>,
    query_pairs: Vec<(&'a str, &'a str)>,
}

impl<'a, S: Serialize> Request<'a, S> {
    pub fn new(bridge: &'a Bridge, request_type: RequestType<S>) -> Self {
        Self {
            bridge,
            request_type,
            custom_headers: vec![],
            path: None,
            query_pairs: vec![],
        }
    }

    pub fn with_custom_headers(self, headers: Vec<(HeaderName, HeaderValue)>) -> Self {
        Self {
            custom_headers: headers,
            ..self
        }
    }

    pub fn to(self, path: &'a str) -> Self {
        Self {
            path: Some(path),
            ..self
        }
    }

    pub fn with_query_pair(self, name: &'a str, value: &'a str) -> Self {
        let mut query_pairs = self.query_pairs;
        query_pairs.push((name, value));
        Self {
            query_pairs,
            ..self
        }
    }

    /// issue the external request by consuming the bridge request and
    /// returning a [Response](struct.Response.html)
    pub fn send<T>(self, response_extractor: &[&str]) -> BridgeRsResult<Response<T>>
    where
        for<'de> T: Deserialize<'de> + Debug,
        Self: Sized,
    {
        let request = self.get_request_type();

        let request_builder = self
            .get_client()
            .request(request.get_method(), self.get_url().as_str())
            .header(CONTENT_TYPE, "application/json")
            .header(
                HeaderName::from_static("x-request-id"),
                &request.id().to_string(),
            );
        let request_builder = self
            .custom_headers()
            .iter()
            .fold(request_builder, |request, (name, value)| {
                request.header(name, value)
            });

        let response = request_builder
            .body(request.body_as_string()?)
            .send()
            .map_err(|e| BridgeRsError::HttpError {
                url: self.get_url(),
                source: e,
            })?;
        let status_code = response.status();
        if !status_code.is_success() {
            return Err(BridgeRsError::WrongStatusCode(self.get_url(), status_code));
        }
        let response_body = response.text().map_err(|e| BridgeRsError::HttpError {
            source: e,
            url: self.get_url(),
        })?;
        let json_value = serde_json::from_str(response_body.as_str()).map_err(|e| {
            BridgeRsError::ResponseBodyNotDeserializable {
                status_code,
                source: e,
            }
        })?;
        let mut selectors = response_extractor.to_vec();
        if request.is_graphql() {
            selectors.insert(0, "data");
        };
        let response_body = extract_inner_json(self.get_url(), selectors, json_value)?;
        Ok(Response::new(
            response_body,
            status_code,
            self.get_request_type().id(),
        ))
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

fn extract_inner_json<T>(url: Url, selectors: Vec<&str>, json_value: Value) -> BridgeRsResult<T>
where
    for<'de> T: Deserialize<'de> + Debug,
{
    let inner_result =
        selectors
            .into_iter()
            .try_fold(&json_value, |acc: &Value, accessor: &str| {
                acc.get(accessor).ok_or_else(|| {
                    BridgeRsError::SelectorNotFound(url.clone(), accessor.to_string(), acc.clone())
                })
            })?;
    Ok(serde_json::from_value::<T>(inner_result.clone())?)
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

    pub fn body_as_string(&self) -> BridgeRsResult<String> {
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
