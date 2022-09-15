use std::convert::TryInto;
use std::time::Duration;
use std::{borrow::Cow, collections::HashSet};

use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    multipart::Form,
    Method, Url,
};
use serde::Serialize;
use uuid::Uuid;

use crate::errors::PrimaBridgeResult;
use crate::request::{Body, DeliverableRequest, DeliverableRequestBody, MultipartFormFileField, RequestType};
use crate::{Bridge, MultipartFile};

/// The RestRequest is a struct that represent a REST request to be done with the [Bridge](./../struct.Bridge.html)
#[derive(Debug)]
pub struct RestRequest<'a> {
    id: Uuid,
    bridge: &'a Bridge,
    body: Option<Body>,
    method: Method,
    timeout: Duration,
    path: Option<&'a str>,
    query_pairs: Vec<(&'a str, &'a str)>,
    ignore_status_code: bool,
    custom_headers: HeaderMap,
    multipart: Option<RestMultipart>,
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
            timeout: Duration::from_secs(60),
            query_pairs: Default::default(),
            ignore_status_code: Default::default(),
            custom_headers: Default::default(),
            multipart: Default::default(),
        }
    }

    /// Request method defaults to `POST`
    pub fn new_with_multipart(bridge: &'a Bridge, multipart: RestMultipart) -> Self {
        Self {
            method: Method::POST,
            id: Uuid::new_v4(),
            bridge,
            body: Default::default(),
            path: Default::default(),
            timeout: Duration::from_secs(60),
            query_pairs: Default::default(),
            ignore_status_code: Default::default(),
            custom_headers: Default::default(),
            multipart: Some(multipart),
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
        Self { query_pairs, ..self }
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

    fn get_custom_headers(&self) -> &HeaderMap {
        &self.custom_headers
    }

    #[cfg(feature = "auth0")]
    fn get_auth0(&self) -> &Option<crate::auth0::Auth0> {
        &self.bridge.auth0_opt
    }

    fn into_body(self) -> PrimaBridgeResult<DeliverableRequestBody> {
        Ok(match self.multipart {
            Some(multipart) => DeliverableRequestBody::Multipart(multipart.into_form()?),
            None => self
                .body
                .map(Into::into)
                .map(DeliverableRequestBody::Bytes)
                .unwrap_or_default(),
        })
    }

    fn get_request_type(&self) -> RequestType {
        RequestType::Rest
    }
}

#[derive(Debug)]
pub enum RestMultipart {
    Single(MultipartFormFileField),
    Multiple(HashSet<MultipartFormFileField>),
}
impl RestMultipart {
    pub fn single<S>(form_field: S, file: MultipartFile) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::Single(MultipartFormFileField::new(form_field, file))
    }

    pub fn multiple(files: HashSet<MultipartFormFileField>) -> Self {
        Self::Multiple(files)
    }

    fn into_form(self) -> PrimaBridgeResult<Form> {
        let mut form = Form::new();

        match self {
            Self::Single(field) => {
                form = form.part(field.field_name, field.file.into_part()?);
            }

            Self::Multiple(fields) => {
                for field in fields {
                    form = form.part(field.field_name, field.file.into_part()?);
                }
            }
        }

        Ok(form)
    }
}
