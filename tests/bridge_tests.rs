use crate::common::*;
use bridge::prelude::*;
use reqwest::Method;
use serde::Deserialize;
use std::error::Error;

mod common;

#[cfg(feature = "blocking")]
mod blocking;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Data {
    hello: String,
}

#[tokio::test]
async fn simple_request() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(200, "{\"hello\": \"world!\"}");
    let body: Option<String> = None;

    let result: String = bridge
        .request(RequestType::rest(body, Method::GET))
        .send()
        .await?
        .get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[tokio::test]
async fn unserializable_response() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(200, "{\"hello\": \"world!\"}");
    let body: Option<String> = None;

    let result: BridgeRsResult<Response> = bridge
        .request(RequestType::rest(body, Method::GET))
        .send()
        .await;
    assert!(result.is_ok());
    let result: BridgeRsResult<Data> = result?.get_data(&["some_strange_selector"]);
    assert!(result.is_err());
    let error_str = result.err().map(|e| e.to_string()).unwrap();
    assert!(error_str.contains("the data for key `some_strange_selector`"));

    Ok(())
}

#[tokio::test]
async fn simple_request_with_custom_path_and_base_path() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) =
        create_bridge_with_base_and_path(200, "{\"hello\": \"world!\"}", "api", "test_path");
    let body: Option<String> = None;

    let result: String = bridge
        .request(RequestType::rest(body, Method::GET))
        .to("test_path")
        .send()
        .await?
        .get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}
