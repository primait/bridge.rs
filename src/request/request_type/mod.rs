pub use graphql::{GraphQLMultipart, GraphQLRequest};
use reqwest::Method;
pub use rest::{RestMultipart, RestRequest};
use serde::Serialize;

use crate::{errors::PrimaBridgeResult, request::GraphQLBody, Bridge};

use super::DeliverableRequest;

mod graphql;
mod rest;

/// A utility type to construct requests more easily.
pub struct Request;

impl Request {
    /// Create a new GraphQL request
    pub fn graphql<S: Serialize>(
        bridge: &Bridge,
        graphql_body: impl Into<GraphQLBody<S>>,
    ) -> PrimaBridgeResult<GraphQLRequest> {
        GraphQLRequest::new(bridge, graphql_body)
    }

    /// Create a new REST request
    pub fn rest(bridge: &Bridge) -> RestRequest {
        Self::get(bridge)
    }

    /// Create a new GET REST request
    pub fn get(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge)
    }

    /// Create a new POST REST request
    pub fn post(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::POST)
    }

    /// Create a new PATCH REST request
    pub fn patch(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::PATCH)
    }

    /// Create a new DELETE REST request
    pub fn delete(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::DELETE)
    }

    /// Create a new PUT REST request
    pub fn put(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::PUT)
    }
}
