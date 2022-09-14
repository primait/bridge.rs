pub use graphql::GraphQLRequest;
pub use graphql::GraphQLMultipart;
use reqwest::Method;
pub use rest::{RestMultipart, RestRequest};
use serde::Serialize;

use crate::errors::PrimaBridgeResult;
use crate::request::GraphQLBody;
use crate::Bridge;

use super::DeliverableRequest;

mod graphql;
mod rest;

/// A server request
pub struct Request;

/// Useful type to construct new requests
impl Request {
    /// Create a new GraphQL request
    pub fn graphql<S: Serialize>(
        bridge: &Bridge,
        graphql_body: impl Into<GraphQLBody<S>>,
    ) -> PrimaBridgeResult<GraphQLRequest> {
        GraphQLRequest::new(bridge, graphql_body)
    }

    /// Create a rest request
    pub fn rest(bridge: &Bridge) -> RestRequest {
        Self::get(bridge)
    }

    /// Create a new GET request
    pub fn get(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge)
    }

    /// Create a new POST request
    pub fn post(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::POST)
    }

    /// Create a new PATCH request
    pub fn patch(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::PATCH)
    }

    /// Create a new DELETE request
    pub fn delete(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::DELETE)
    }

    /// Create a new PUT request
    pub fn put(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::PUT)
    }
}
