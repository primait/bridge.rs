use std::fmt::Debug;

use reqwest::{StatusCode, Url};
use uuid::Uuid;

use crate::prelude::*;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, PartialEq)]
enum RequestType {
    Rest,
    GraphQL,
}

#[derive(Debug)]
pub struct Response {
    url: String,
    response_body: String,
    status_code: StatusCode,
    request_id: Uuid,
    request_type: RequestType,
}

impl Response {
    pub fn rest(
        url: String,
        response_body: String,
        status_code: StatusCode,
        request_id: Uuid,
    ) -> Self {
        Self {
            url,
            response_body,
            status_code,
            request_id,
            request_type: RequestType::Rest,
        }
    }

    pub fn graphql(
        url: String,
        response_body: String,
        status_code: StatusCode,
        request_id: Uuid,
    ) -> Self {
        Self {
            url,
            response_body,
            status_code,
            request_id,
            request_type: RequestType::GraphQL,
        }
    }

    fn is_graphql(&self) -> bool {
        self.request_type == RequestType::GraphQL
    }

    pub fn get_data<T>(self, response_extractor: &[&str]) -> BridgeRsResult<T>
    where
        for<'de> T: Deserialize<'de> + Debug,
    {
        let json_value = serde_json::from_str(self.response_body.as_str()).map_err(|e| {
            BridgeRsError::ResponseBodyNotDeserializable {
                status_code: self.status_code,
                source: e,
            }
        })?;
        let mut selectors = response_extractor.to_vec();
        if self.is_graphql() {
            selectors.insert(0, "data");
        };
        Ok(extract_inner_json(self.url, selectors, json_value)?)
    }

    pub fn is_ok(&self) -> bool {
        self.status_code.is_success()
    }
}

fn extract_inner_json<T>(url: String, selectors: Vec<&str>, json_value: Value) -> BridgeRsResult<T>
where
    for<'de> T: Deserialize<'de> + Debug,
{
    let inner_result =
        selectors
            .into_iter()
            .try_fold(&json_value, |acc: &Value, accessor: &str| {
                acc.get(accessor).ok_or_else(|| {
                    BridgeRsError::SelectorNotFound(url, accessor.to_string(), acc.clone())
                })
            })?;
    Ok(serde_json::from_value::<T>(inner_result.clone())?)
}
