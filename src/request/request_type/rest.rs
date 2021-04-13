use std::convert::TryInto;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::{Method, Url};
use serde::Serialize;
use uuid::Uuid;

use crate::errors::PrimaBridgeResult;
use crate::request::{Body, DeliverableRequest, RequestType};
use crate::Bridge;

/// The RestRequest is a struct that represent a REST request to be done with the [Bridge](./../struct.Bridge.html)
#[derive(Debug)]
pub struct RestRequest<'a> {
    id: Uuid,
    bridge: &'a Bridge,
    body: Option<Body>,
    method: Method,
    path: Option<&'a str>,
    query_pairs: Vec<(&'a str, &'a str)>,
    ignore_status_code: bool,
    custom_headers: HeaderMap,
}

impl<'a> RestRequest<'a> {
    /// creates a new RestRequest
    pub fn new(bridge: &'a Bridge) -> Self {
        Self {
            id: Uuid::new_v4(),
            bridge,
            body: Default::default(),
            method: Default::default(), // GET
            path: Default::default(),
            query_pairs: Default::default(),
            ignore_status_code: Default::default(),
            custom_headers: Default::default(),
        }
    }
}

#[async_trait]
impl<'a> DeliverableRequest<'a> for RestRequest<'a> {
    fn raw_body(self, body: impl Into<Body>) -> Self {
        Self {
            body: Some(body.into()),
            ..self
        }
    }

    fn json_body<B: Serialize>(self, body: &B) -> PrimaBridgeResult<Self> {
        let mut custom_headers = self.custom_headers;
        custom_headers.append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(Self {
            body: Some(serde_json::to_string(body)?.try_into()?),
            custom_headers,
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

    fn set_custom_headers(self, headers: HeaderMap) -> Self {
        Self {
            custom_headers: headers,
            ..self
        }
    }

    fn set_query_pairs(self, query_pairs: Vec<(&'a str, &'a str)>) -> Self {
        Self {
            query_pairs,
            ..self
        }
    }

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_bridge(&self) -> &Bridge {
        self.bridge
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

    fn get_method(&self) -> Method {
        self.method.clone()
    }

    fn get_custom_headers(&self) -> HeaderMap {
        self.custom_headers.clone()
    }

    fn get_body(&self) -> Vec<u8> {
        self.body.clone().map(Into::into).unwrap_or_default()
    }

    fn get_request_type(&self) -> RequestType {
        RequestType::Rest
    }
}
