use std::error::Error;

use serde::Deserialize;

use crate::common::*;
use prima_bridge::prelude::*;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Data {
    hello: String,
}

#[test]
fn simple_request() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(200, "{\"hello\": \"world!\"}");

    let result: String = RestRequest::new(&bridge).send()?.get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[test]
fn simple_request_with_custom_path() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge_with_path(200, "{\"hello\": \"world!\"}", "/test_path");

    let result: String = RestRequest::new(&bridge)
        .to("test_path")
        .send()?
        .get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[test]
fn simple_request_with_custom_path_and_base_path() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) =
        create_bridge_with_base_and_path(200, "{\"hello\": \"world!\"}", "api", "test_path");

    let result: String = RestRequest::new(&bridge)
        .to("test_path")
        .send()?
        .get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[test]
fn simple_request_with_custom_sub_path() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) =
        create_bridge_with_path(200, "{\"hello\": \"world!\"}", "/test_path/test_subpath");

    let result: String = RestRequest::new(&bridge)
        .to("/test_path/test_subpath")
        .send()?
        .get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[test]
fn unserializable_response() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(200, "{\"hello\": \"world!\"}");

    let result: PrimaBridgeResult<Response> = RestRequest::new(&bridge).send();
    assert!(result.is_ok());
    let result: PrimaBridgeResult<Data> = result?.get_data(&["some_strange_selector"]);
    assert!(result.is_err());
    let error_str = result.err().map(|e| e.to_string()).unwrap();
    assert!(error_str.contains("the data for key `some_strange_selector`"));

    Ok(())
}

#[test]
fn wrong_status_code() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(400, "{\"hello\": \"world!\"}");
    let result: PrimaBridgeResult<Response> = RestRequest::new(&bridge).send();

    assert!(result.is_err());
    let error_str = result.err().map(|e| e.to_string()).unwrap();
    assert!(error_str.contains("400 Bad Request"));

    Ok(())
}

#[test]
fn response_body_not_deserializable() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(201, "{\"hello\": \"worl______________}");

    let result: PrimaBridgeResult<Response> = RestRequest::new(&bridge).send();
    assert!(result.is_ok());
    let result: PrimaBridgeResult<Data> = result?.get_data(&["hello"]);
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

    let result = RestRequest::new(&bridge).send();
    assert!(result.is_ok());
    let result: PrimaBridgeResult<Data> = result?.get_data(&["hello"]);
    assert!(result.is_err());
    assert_eq!(
        result.err().map(|e| e.to_string()),
        Some("unserializable body. response status code: 204 No Content, error: EOF while parsing a value at line 1 column 0".to_string())
    );

    Ok(())
}

#[test]
fn simple_request_with_custom_headers() -> Result<(), Box<dyn Error>> {
    let header = ("header", "custom");
    let path = "/test_path/simple_request_with_custom_headers";

    let (_m, bridge) =
        create_bridge_with_path_and_header(200, "{\"hello\": \"world!\"}", path, header);

    let response = RestRequest::new(&bridge).to(path).send()?;

    let custom = response
        .headers()
        .get(header.0)
        .expect("It should contain the custom header")
        .to_str()?;

    assert_eq!(header.1, custom);

    let result: String = response.get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[test]
fn simple_request_with_wrong_status_code() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(400, "{\"hello\": \"world!\"}");

    let result: String = RestRequest::new(&bridge)
        .ignore_status_code()
        .send()?
        .get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[test]
fn simple_request_with_wrong_status_code_and_wrong_body() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(400, "{\"hello\": \"worl______________}");

    let request = RestRequest::new(&bridge).ignore_status_code();

    let result = request.send();
    assert!(result.is_ok());
    let result: PrimaBridgeResult<Data> = result?.get_data(&["hello"]);
    assert!(result.is_err());
    //let error_str = result.err().map(|e| e.to_string()).unwrap();
    assert_eq!(
        result.err().map(|e| e.to_string()),
        Some("unserializable body. response status code: 400 Bad Request, error: EOF while parsing a string at line 1 column 30".to_string())
    );

    Ok(())
}

#[test]
fn request_with_custom_body() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge_with_body_matcher("abcde");
    let result = RestRequest::new(&bridge).raw_body("abcde")?.send();
    assert!(result.is_ok());
    Ok(())
}
