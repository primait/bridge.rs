use reqwest::Method;
use serde::Serialize;

pub use graphql::GraphQLRequest;
pub use rest::RestRequest;

use super::DeliverableRequest;
use crate::errors::PrimaBridgeResult;
use crate::request::GraphQLBody;
use crate::Bridge;

mod graphql;
mod rest;

pub struct Request;

impl Request {
    pub fn graphql<S: Serialize>(
        bridge: &Bridge,
        graphql_body: impl Into<GraphQLBody<S>>,
    ) -> PrimaBridgeResult<GraphQLRequest> {
        GraphQLRequest::new(bridge, graphql_body)
    }

    pub fn get(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge)
    }

    pub fn post(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::POST)
    }

    pub fn patch(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::PATCH)
    }

    pub fn delete(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::DELETE)
    }

    pub fn put(bridge: &Bridge) -> RestRequest {
        RestRequest::new(bridge).method(Method::PUT)
    }
}
