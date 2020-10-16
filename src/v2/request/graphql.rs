use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::request::GraphQLBody;
use crate::v2::{Body, DeliverableRequest, RequestType};
use crate::{Bridge, Response};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, Url};
use serde::Serialize;
use std::convert::TryInto;
use uuid::Uuid;

pub struct GraphQLRequest<'a> {
    pub id: Uuid,
    bridge: &'a Bridge,
    body: Option<Body>,
    method: Method,
    path: Option<&'a str>,
    query_pairs: Vec<(&'a str, &'a str)>,
    ignore_status_code: bool,
    custom_headers: Vec<(HeaderName, HeaderValue)>,
}

impl<'a> GraphQLRequest<'a> {
    pub fn new<S: Serialize>(
        bridge: &'a Bridge,
        graphql_body: GraphQLBody<S>,
    ) -> PrimaBridgeResult<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            bridge,
            body: Some(serde_json::to_string(&graphql_body)?.try_into()?),
            method: Default::default(), // GET
            path: Default::default(),
            query_pairs: Default::default(),
            ignore_status_code: Default::default(),
            custom_headers: Default::default(),
        })
    }
}

impl<'a> DeliverableRequest<'a> for GraphQLRequest<'a> {
    fn send(&self) -> PrimaBridgeResult<Response> {
        unimplemented!()
    }

    fn raw_body(
        self,
        body: impl TryInto<Body, Error = PrimaBridgeError>,
    ) -> PrimaBridgeResult<Self> {
        Ok(Self {
            body: Some(body.try_into()?),
            ..self
        })
    }

    fn json_body<B: Serialize>(self, body: B) -> PrimaBridgeResult<Self> {
        Ok(Self {
            body: Some(serde_json::to_string(&body)?.try_into()?),
            ..self
        })
    }

    fn method(self, method: Method) -> Self {
        Self { method, ..self }
    }

    fn to(self, path: &'a str) -> Self {
        Self {
            path: Some(path),
            ..self
        }
    }

    fn ignore_status_code(self) -> Self {
        Self {
            ignore_status_code: true,
            ..self
        }
    }

    fn with_custom_headers(self, headers: Vec<(HeaderName, HeaderValue)>) -> Self {
        Self {
            custom_headers: headers,
            ..self
        }
    }

    fn get_path(&self) -> Option<&str> {
        self.path
    }

    fn endpoint(&self) -> Url {
        self.bridge.endpoint.clone()
    }

    fn get_query_pairs(&self) -> &[(&'a str, &'a str)] {
        self.query_pairs.as_slice()
    }

    fn get_ignore_status_code(&self) -> bool {
        self.ignore_status_code
    }

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_bridge(&self) -> &Bridge {
        self.bridge
    }

    fn get_method(&self) -> Method {
        self.method.clone()
    }

    fn get_custom_headers(&self) -> &[(HeaderName, HeaderValue)] {
        self.custom_headers.as_slice()
    }

    fn get_body(&self) -> PrimaBridgeResult<Vec<u8>> {
        unimplemented!()
    }

    fn get_request_type(&self) -> RequestType {
        RequestType::GraphQL
    }
}
