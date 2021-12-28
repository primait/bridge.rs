use std::fmt::Debug;

use reqwest::{header::HeaderMap, StatusCode, Url};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::prelude::*;
use crate::response::graphql::{ParsedGraphqlResponse, ParsedGraphqlResponseExt};

pub mod graphql;

#[derive(Debug, PartialEq)]
enum RequestType {
    Rest,
    #[allow(clippy::upper_case_acronyms)]
    GraphQL,
}

/// A server response
#[derive(Debug)]
pub struct Response {
    url: Url,
    response_body: Vec<u8>,
    status_code: StatusCode,
    response_headers: HeaderMap,
    request_id: Uuid,
    request_type: RequestType,
}

impl Response {
    #[doc(hidden)]
    pub fn rest(
        url: Url,
        response_body: Vec<u8>,
        status_code: StatusCode,
        response_headers: HeaderMap,
        request_id: Uuid,
    ) -> Self {
        Self {
            url,
            response_body,
            status_code,
            response_headers,
            request_id,
            request_type: RequestType::Rest,
        }
    }

    #[doc(hidden)]
    pub fn graphql(
        url: Url,
        response_body: Vec<u8>,
        status_code: StatusCode,
        response_headers: HeaderMap,
        request_id: Uuid,
    ) -> Self {
        Self {
            url,
            response_body,
            status_code,
            response_headers,
            request_id,
            request_type: RequestType::GraphQL,
        }
    }

    #[doc(hidden)]
    fn is_graphql(&self) -> bool {
        self.request_type == RequestType::GraphQL
    }

    /// Returns an `HeaderMap` of response headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.response_headers
    }

    /// Returns data from the function.
    pub fn get_data<T>(self, response_extractor: &[&str]) -> PrimaBridgeResult<T>
    where
        for<'de> T: Deserialize<'de> + Debug,
    {
        let json_value = serde_json::from_slice(&self.response_body[..]).map_err(|e| {
            PrimaBridgeError::ResponseBodyNotDeserializable {
                status_code: self.status_code,
                source: e,
            }
        })?;
        let mut selectors = response_extractor.to_vec();
        if self.is_graphql() {
            selectors.insert(0, "data");
        };
        extract_inner_json(self.url, selectors, json_value)
    }

    /// This functions return a Result with a [ParsedGraphqlResponse]
    /// Look at the type documentation for more specifications
    pub fn parse_graphql_response<T>(&self) -> PrimaBridgeResult<ParsedGraphqlResponse<T>>
    where
        for<'de> T: Deserialize<'de>,
    {
        ParsedGraphqlResponse::from_str(
            std::str::from_utf8(self.raw_body()).map_err(PrimaBridgeError::utf8_error)?,
        )
    }

    pub fn raw_body(&self) -> &Vec<u8> {
        &self.response_body
    }

    /// returns `true` if the response is successful
    pub fn is_ok(&self) -> bool {
        self.status_code.is_success()
    }
}

fn extract_inner_json<T>(url: Url, selectors: Vec<&str>, json_value: Value) -> PrimaBridgeResult<T>
where
    for<'de> T: Deserialize<'de> + Debug,
{
    let inner_result =
        selectors
            .into_iter()
            .try_fold(&json_value, |acc: &Value, accessor: &str| {
                acc.get(accessor).ok_or_else(|| {
                    PrimaBridgeError::SelectorNotFound(
                        url.clone(),
                        accessor.to_string(),
                        acc.clone(),
                    )
                })
            })?;
    Ok(serde_json::from_value::<T>(inner_result.clone())?)
}
