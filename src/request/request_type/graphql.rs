use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::{Method, Url};
use serde::Serialize;
use uuid::Uuid;

use crate::errors::PrimaBridgeResult;
use crate::request::{Body, DeliverableRequest, GraphQLBody, RequestType};
use crate::Bridge;

/// The GraphQLRequest is a struct that represent a GraphQL request to be done with the [Bridge](./../struct.Bridge.html)
#[allow(clippy::upper_case_acronyms)]
pub struct GraphQLRequest<'a> {
    id: Uuid,
    bridge: &'a Bridge,
    body: Option<Body>,
    method: Method,
    timeout: Duration,
    path: Option<&'a str>,
    query_pairs: Vec<(&'a str, &'a str)>,
    ignore_status_code: bool,
    custom_headers: HeaderMap,
    uploads: HashMap<String, Vec<u8>>,
}

impl<'a> GraphQLRequest<'a> {
    /// creates a new GraphQLRequest
    pub fn new<S: Serialize>(
        bridge: &'a Bridge,
        graphql_body: impl Into<GraphQLBody<S>>,
    ) -> PrimaBridgeResult<Self> {
        let mut custom_headers = HeaderMap::default();
        custom_headers.append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(Self {
            id: Uuid::new_v4(),
            bridge,
            body: Some(serde_json::to_string(&graphql_body.into())?.try_into()?),
            method: Method::POST,
            path: Default::default(),
            timeout: Duration::from_secs(60),
            query_pairs: Default::default(),
            ignore_status_code: Default::default(),
            custom_headers,
            uploads: HashMap::new(),
        })
    }

    pub fn new_with_uploads<S: Serialize>(
        bridge: &'a Bridge,
        graphql_body: impl Into<GraphQLBody<S>>,
        uploads: HashMap<String, Vec<u8>>,
    ) -> PrimaBridgeResult<Self> {
        let mut custom_headers = HeaderMap::default();
        custom_headers.append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(Self {
            id: Uuid::new_v4(),
            bridge,
            body: Some(serde_json::to_string(&graphql_body.into())?.try_into()?),
            method: Method::POST,
            path: Default::default(),
            timeout: Duration::from_secs(60),
            query_pairs: Default::default(),
            ignore_status_code: Default::default(),
            custom_headers,
            uploads,
        })
    }
}

#[async_trait]
impl<'a> DeliverableRequest<'a> for GraphQLRequest<'a> {
    fn raw_body(self, body: impl Into<Body>) -> Self {
        Self {
            body: Some(body.into()),
            ..self
        }
    }

    fn json_body<B: Serialize>(self, body: &B) -> PrimaBridgeResult<Self> {
        Ok(Self {
            body: Some(serde_json::to_string(body)?.try_into()?),
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

    fn set_timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }

    fn get_timeout(&self) -> Duration {
        self.timeout
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

    #[cfg(feature = "auth0")]
    fn get_auth0(&self) -> &Option<crate::auth0::Auth0> {
        &self.bridge.auth0_opt
    }

    fn get_body(&self) -> Vec<u8> {
        self.body.clone().map(Into::into).unwrap_or_default()
    }

    fn get_request_type(&self) -> RequestType {
        RequestType::GraphQL
    }

    fn get_form(&self) -> Option<reqwest::multipart::Form> {
        if self.uploads.is_empty() {
            None
        } else {
            let mut form = reqwest::multipart::Form::new();
            form = form.text("operations", String::from(self.body.as_ref().unwrap()));

            let mut map = HashMap::<String, Vec<String>>::new();
            for (index, (path, file)) in self.uploads.iter().enumerate() {
                map.insert(index.to_string(), vec![path.clone()]);
                form = form.part(
                    index.to_string(),
                    reqwest::multipart::Part::bytes(file.to_vec()),
                );
            }
            println!("{}", serde_json::to_string(&map).unwrap());
            form = form.text("map", serde_json::to_string(&map).unwrap());

            Some(form)
        }
    }
}
