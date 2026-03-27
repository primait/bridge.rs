use std::fmt::Debug;

use reqwest::{header::HeaderMap, StatusCode, Url};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::errors::BodyStructure;
use crate::prelude::*;
use crate::response::graphql::{ParsedGraphqlResponse, ParsedGraphqlResponseExt};

pub mod graphql;

#[derive(Debug, PartialEq)]
enum RequestType {
    Rest,
    #[allow(clippy::upper_case_acronyms)]
    GraphQL,
}

/// A server response.
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
        ParsedGraphqlResponse::from_str(std::str::from_utf8(self.raw_body()).map_err(PrimaBridgeError::utf8_error)?)
    }

    pub fn raw_body(&self) -> &Vec<u8> {
        &self.response_body
    }

    pub fn status_code(&self) -> &StatusCode {
        &self.status_code
    }

    /// returns `true` if the response is successful
    pub fn is_ok(&self) -> bool {
        self.status_code.is_success()
    }

    pub fn request_id(&self) -> Uuid {
        self.request_id
    }
}

fn extract_inner_json<T>(url: Url, selectors: Vec<&str>, json_value: Value) -> PrimaBridgeResult<T>
where
    for<'de> T: Deserialize<'de> + Debug,
{
    let inner_result = selectors
        .into_iter()
        .try_fold(&json_value, |acc: &Value, accessor: &str| {
            acc.get(accessor).ok_or_else(|| {
                PrimaBridgeError::SelectorNotFound(Box::new((url.clone(), accessor.to_string(), acc.clone())))
            })
        })?;
    serde_path_to_error::deserialize(inner_result.clone()).map_err(|error| {
        let body_structure = BodyStructure::at_path(inner_result, error.path());
        PrimaBridgeError::DeserializationError { body_structure, error }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderMap;
    use serde::Deserialize;

    fn rest_response(body: &str) -> Response {
        Response::rest(
            Url::parse("http://test.example.com").unwrap(),
            body.as_bytes().to_vec(),
            StatusCode::OK,
            HeaderMap::new(),
            Uuid::new_v4(),
        )
    }

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    #[allow(dead_code)]
    enum PaymentResponse {
        Success { transaction_id: String, inner: Inner },
        Error { error_code: String, message: String },
    }

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Inner {
        secret: String,
    }

    #[test]
    fn get_data_deserializes_matching_variant() {
        let resp = rest_response(r#"{"data": {"transaction_id": "tx_123", "inner": {"secret": "secret_value"}}}"#);
        let result: PaymentResponse = resp.get_data(&["data"]).unwrap();
        assert!(matches!(result, PaymentResponse::Success { transaction_id, .. } if transaction_id == "tx_123"));
    }

    #[test]
    fn get_data_reports_deserialization_error_on_untagged_enum_mismatch() {
        let resp = rest_response(r#"{"data": {"unexpected_field": 42}}"#);
        let err = resp.get_data::<PaymentResponse>(&["data"]).unwrap_err();

        if let PrimaBridgeError::DeserializationError { error, .. } = &err {
            let msg = error.to_string();
            assert!(
                msg.contains("did not match any variant of untagged enum"),
                "expected untagged enum message, got: {msg}"
            );
            // at the top level, the path is "." (root)
            let path = error.path().to_string();
            assert_eq!(path, ".", "error path should be root");
        } else {
            panic!("expected DeserializationError, got: {err:?}");
        }
    }

    #[test]
    fn get_data_includes_body_structure_without_values() {
        let resp = rest_response(r#"{"data": {"unexpected_field": 42}}"#);
        let err = resp.get_data::<PaymentResponse>(&["data"]).unwrap_err();

        if let PrimaBridgeError::DeserializationError { body_structure, .. } = &err {
            let body_structure = body_structure.to_string();
            assert!(
                body_structure.contains("unexpected_field"),
                "body_structure should contain the field name: {body_structure}"
            );
            assert!(
                !body_structure.contains("42"),
                "body_structure should not contain actual values: {body_structure}"
            );
        } else {
            panic!("expected DeserializationError, got: {err:?}");
        }
    }

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Outer {
        payment: PaymentResponse,
        status: String,
    }

    #[test]
    fn get_data_reports_nested_path_on_untagged_enum_mismatch() {
        let resp = rest_response(
            r#"{"data": {"payment": {"unexpected_field": 42, "transaction_id": "tx_123"}, "status": "ok"}}"#,
        );
        let err = resp.get_data::<Outer>(&["data"]).unwrap_err();

        if let PrimaBridgeError::DeserializationError {
            error, body_structure, ..
        } = &err
        {
            let path = error.path().to_string();
            let body_structure = body_structure.to_string();
            assert_eq!(path, "payment", "error path should point to 'payment' field");
            assert!(
                body_structure.contains("unexpected_field"),
                "body_structure should show structure at error path: {body_structure}"
            );
        } else {
            panic!("expected DeserializationError, got: {err:?}");
        }
    }
}
