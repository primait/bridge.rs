pub use graphql::GraphQLMultipart;
pub use graphql::GraphQLRequest;
use reqwest::Method;
pub use rest::{RestMultipart, RestRequest};
use serde::Serialize;

use crate::errors::PrimaBridgeResult;
use crate::request::GraphQLBody;
use crate::BridgeClient;
use crate::BridgeImpl;

use super::DeliverableRequest;

mod graphql;
mod rest;

/// A utility type to construct requests more easily.
pub struct Request;

impl Request {
    /// Create a new GraphQL request
    pub fn graphql<S: Serialize, Client: BridgeClient>(
        bridge: &BridgeImpl<Client>,
        graphql_body: impl Into<GraphQLBody<S>>,
    ) -> PrimaBridgeResult<GraphQLRequest<Client>> {
        GraphQLRequest::new(bridge, graphql_body)
    }

    /// Create a new REST request
    pub fn rest<Client: BridgeClient>(bridge: &BridgeImpl<Client>) -> RestRequest<Client> {
        Self::get(bridge)
    }

    /// Create a new GET REST request
    pub fn get<Client: BridgeClient>(bridge: &BridgeImpl<Client>) -> RestRequest<Client> {
        RestRequest::new(bridge)
    }

    /// Create a new POST REST request
    pub fn post<Client: BridgeClient>(bridge: &BridgeImpl<Client>) -> RestRequest<Client> {
        RestRequest::new(bridge).method(Method::POST)
    }

    /// Create a new PATCH REST request
    pub fn patch<Client: BridgeClient>(bridge: &BridgeImpl<Client>) -> RestRequest<Client> {
        RestRequest::new(bridge).method(Method::PATCH)
    }

    /// Create a new DELETE REST request
    pub fn delete<Client: BridgeClient>(bridge: &BridgeImpl<Client>) -> RestRequest<Client> {
        RestRequest::new(bridge).method(Method::DELETE)
    }

    /// Create a new PUT REST request
    pub fn put<Client: BridgeClient>(bridge: &BridgeImpl<Client>) -> RestRequest<Client> {
        RestRequest::new(bridge).method(Method::PUT)
    }
}
