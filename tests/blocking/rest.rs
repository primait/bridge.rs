use std::error::Error;

use reqwest::Method;
use serde::Deserialize;

use crate::common::*;
use bridge::prelude::*;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Data {
    hello: String,
}

#[test]
fn simple_request() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(200, "{\"hello\": \"world!\"}");
    let body: Option<String> = None;

    let result: String = bridge
        .request(RequestType::rest(body, Method::GET))
        .send(&["hello"])?
        .get_data()?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[test]
fn simple_request_with_custom_path() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge_with_path(200, "{\"hello\": \"world!\"}", "/test_path");
    let body: Option<String> = None;

    let result: String = bridge
        .request(RequestType::rest(body, Method::GET))
        .to("test_path")
        .send(&["hello"])?
        .get_data()?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[test]
fn simple_request_with_custom_path_and_base_path() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) =
        create_bridge_with_base_and_path(200, "{\"hello\": \"world!\"}", "api", "test_path");
    let body: Option<String> = None;

    let result: String = bridge
        .request(RequestType::rest(body, Method::GET))
        .to("test_path")
        .send(&["hello"])?
        .get_data()?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[test]
fn simple_request_with_custom_sub_path() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) =
        create_bridge_with_path(200, "{\"hello\": \"world!\"}", "/test_path/test_subpath");
    let body: Option<String> = None;

    let result: String = bridge
        .request(RequestType::rest(body, Method::GET))
        .to("/test_path/test_subpath")
        .send(&["hello"])?
        .get_data()?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[test]
fn unserializable_response() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(200, "{\"hello\": \"world!\"}");
    let body: Option<String> = None;

    let result: BridgeRsResult<Response<Data>> = bridge
        .request(RequestType::rest(body, Method::GET))
        .send(&["some_strange_selector"]);

    assert!(result.is_err());
    let error_str = result.err().map(|e| e.to_string()).unwrap();
    assert!(error_str.contains("the data for key `some_strange_selector`"));

    Ok(())
}

#[test]
fn wrong_status_code() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(400, "{\"hello\": \"world!\"}");
    let body: Option<String> = None;

    let result: BridgeRsResult<Response<Data>> = bridge
        .request(RequestType::rest(body, Method::GET))
        .send(&[]);

    assert!(result.is_err());
    let error_str = result.err().map(|e| e.to_string()).unwrap();
    assert!(error_str.contains("400 Bad Request"));

    Ok(())
}

#[test]
fn response_body_not_deserializable() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(201, "{\"hello\": \"worl______________}");
    let body: Option<String> = None;

    let result: BridgeRsResult<Response<Data>> = bridge
        .request(RequestType::rest(body, Method::GET))
        .send(&["hello"]);

    assert!(result.is_err());
    //let error_str = result.err().map(|e| e.to_string()).unwrap();
    assert_eq!(
        result.err().map(|e| e.to_string()),
        Some("unserializable body. response status code: 201 Created, error: EOF while parsing a string at line 1 column 30".to_string())
    );

    Ok(())
}

#[test]
fn response_with_empty_body() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(204, "");
    let body: Option<String> = None;

    let result: BridgeRsResult<Response<Data>> = bridge
        .request(RequestType::rest(body, Method::GET))
        .send(&["hello"]);

    assert!(result.is_err());
    assert_eq!(
        result.err().map(|e| e.to_string()),
        Some("unserializable body. response status code: 204 No Content, error: EOF while parsing a value at line 1 column 0".to_string())
    );

    Ok(())
}
