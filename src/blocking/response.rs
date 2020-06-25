use std::fmt::Debug;

use reqwest::StatusCode;
use uuid::Uuid;

use crate::prelude::*;

#[derive(Debug)]
pub struct Response<T> {
    response_body: Option<T>,
    status_code: StatusCode,
    request_id: Uuid,
}

impl<T: Debug> Response<T> {
    pub fn new(response_body: Option<T>, status_code: StatusCode, request_id: Uuid) -> Self {
        Self {
            response_body,
            status_code,
            request_id,
        }
    }

    pub fn get_data(self) -> BridgeRsResult<T> {
        match self.response_body {
            None => Err(BridgeRsError::EmptyBody),
            Some(body) => Ok(body),
        }
    }

    pub fn is_ok(&self) -> bool {
        self.status_code.is_success()
    }
}

impl<T: Debug> From<Response<T>> for BridgeRsResult<T> {
    fn from(response: Response<T>) -> Self {
        response.get_data()
    }
}
