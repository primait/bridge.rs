use self::request::{Request, RequestType};
use reqwest::Url;
use serde::Serialize;

pub mod request;
pub mod response;

/// The bridge instance to issue external requests.
#[derive(Debug)]
pub struct Bridge {
    client: reqwest::blocking::Client,
    /// the url this bridge should call to
    endpoint: Url,
}

impl Bridge {
    pub fn new(endpoint: Url) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            endpoint,
        }
    }

    pub fn request<S: Serialize>(&self, request_type: RequestType<S>) -> Request<S> {
        Request::new(&self, request_type)
    }
}
