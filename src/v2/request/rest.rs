use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::v2::Body;
use crate::Bridge;
#[cfg(feature = "blocking")]
use reqwest::blocking::Client as ReqwestClient;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Method;
use std::convert::TryInto;
use uuid::Uuid;

pub struct RestRequest<'a> {
    pub id: Uuid,
    pub bridge: &'a Bridge,
    body: Option<Body>,
    method: Method,
}

impl<'a> RestRequest<'a> {
    pub fn new(bridge: &'a Bridge) -> Self {
        Self {
            id: Uuid::new_v4(),
            bridge,
            body: Default::default(),
            method: Default::default(), // GET
        }
    }

    pub fn body(
        self,
        body: impl TryInto<Body, Error = PrimaBridgeError>,
    ) -> PrimaBridgeResult<Self> {
        Ok(Self {
            body: Some(body.try_into()?),
            ..self
        })
    }

    pub fn method(self, method: Method) -> Self {
        Self { method, ..self }
    }

    pub fn get_method(&self) -> Method {
        self.method.clone()
    }

    pub fn get_client(&self) -> &ReqwestClient {
        &self.bridge.client
    }
    pub fn get_body(&self) -> PrimaBridgeResult<Vec<u8>> {
        self.body
            .clone()
            .map(|b| b.try_into())
            .unwrap_or_else(|| Ok(vec![]))
    }
}
