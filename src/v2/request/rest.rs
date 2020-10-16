use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::v2::{Body, DeliverableRequest, RequestType};
use crate::{Bridge, Response};
#[cfg(feature = "blocking")]
use reqwest::blocking::Client as ReqwestClient;
use reqwest::header::{HeaderName, HeaderValue};
#[cfg(not(feature = "blocking"))]
use reqwest::Client as ReqwestClient;
use reqwest::{Method, Url};
use serde::Serialize;
use std::convert::TryInto;
use uuid::Uuid;

pub struct RestRequest<'a> {
    pub id: Uuid,
    pub bridge: &'a Bridge,
    body: Option<Body>,
    method: Method,
    path: Option<&'a str>,
    query_pairs: Vec<(&'a str, &'a str)>,
    ignore_status_code: bool,
    custom_headers: Vec<(HeaderName, HeaderValue)>,
}

impl<'a> RestRequest<'a> {
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

    pub fn raw_body(
        self,
        body: impl TryInto<Body, Error = PrimaBridgeError>,
    ) -> PrimaBridgeResult<Self> {
        Ok(Self {
            body: Some(body.try_into()?),
            ..self
        })
    }

    pub fn json_body<B: Serialize>(self, body: B) -> PrimaBridgeResult<Self> {
        Ok(Self {
            body: Some(serde_json::to_string(&body)?.try_into()?),
            ..self
        })
    }

    pub fn method(self, method: Method) -> Self {
        Self { method, ..self }
    }

    pub fn get_body(&self) -> PrimaBridgeResult<Vec<u8>> {
        self.body
            .clone()
            .map(TryInto::try_into)
            .unwrap_or_else(|| Ok(vec![]))
    }
}

impl<'a> DeliverableRequest<'a> for RestRequest<'a> {
    fn raw_body(
        self,
        body: impl TryInto<Body, Error = PrimaBridgeError>,
    ) -> PrimaBridgeResult<Self> {
        self.raw_body(body)
    }

    fn json_body<B: Serialize>(self, body: B) -> PrimaBridgeResult<Self> {
        self.json_body(body)
    }

    fn method(self, method: Method) -> Self {
        self.method(method)
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

    fn get_custom_headers(&self) -> &[(HeaderName, HeaderValue)] {
        self.custom_headers.as_slice()
    }

    fn get_body(&self) -> PrimaBridgeResult<Vec<u8>> {
        self.get_body()
    }

    fn get_request_type(&self) -> RequestType {
        RequestType::Rest
    }
}
