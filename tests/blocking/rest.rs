use std::error::Error;

use mockito::*;
use reqwest::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;

use prima_bridge::prelude::*;
use prima_bridge::Request;

use crate::common::*;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
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
fn request_with_custom_raw_body() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge_with_raw_body_matcher("abcde");
    let result = RestRequest::new(&bridge).raw_body("abcde").send();
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn request_post_with_custom_raw_body() -> Result<(), Box<dyn Error>> {
    let body = "abcde";
    let _mock = mock("POST", "/")
        .match_body(Matcher::Exact(body.to_owned()))
        .with_status(200)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    let result = Request::post(&bridge).raw_body(body).send();
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn request_with_custom_json_body() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge_with_json_body_matcher(json!({"hello": "world"}));
    let data = Data {
        hello: "world".to_string(),
    };
    let request = RestRequest::new(&bridge).json_body(&data)?;
    assert_eq!(
        request.get_custom_headers().get(CONTENT_TYPE),
        HeaderValue::from_str("application/json").ok().as_ref()
    );
    let result = request.send();

    assert!(result.is_ok());
    Ok(())
}

#[test]
fn request_with_custom_headers() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge_with_header_matcher(("content-type", "application/json"));

    let result = RestRequest::new(&bridge)
        .json_body(&"test".to_owned())?
        .with_custom_headers(vec![(
            HeaderName::from_static("x-prima"),
            HeaderValue::from_static("test-value"),
        )])
        .send()?;
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn request_with_custom_user_agent() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge_with_user_agent("test");

    let result = RestRequest::new(&bridge).send()?;
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn request_with_query_string() -> Result<(), Box<dyn Error>> {
    let _mock = mock("GET", "/")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("hello".into(), "world!".into()),
            Matcher::UrlEncoded("prima".into(), "bridge".into()),
        ]))
        .with_status(200)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    let result = Request::get(&bridge)
        .with_query_pair("hello", "world!")
        .with_query_pair("prima", "bridge")
        .send();
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn request_with_query_string_by_calling_once() -> Result<(), Box<dyn Error>> {
    let _mock = mock("GET", "/")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("hello".into(), "world!".into()),
            Matcher::UrlEncoded("prima".into(), "bridge".into()),
        ]))
        .with_status(200)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    let result = Request::get(&bridge)
        .with_query_pairs(vec![("hello", "world!"), ("prima", "bridge")])
        .send();
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn request_with_binary_raw_body() -> Result<(), Box<dyn Error>> {
    let body = b"abcde";
    let (_m, bridge) = create_bridge_with_binary_body_matcher(body);
    let result = RestRequest::new(&bridge).raw_body(body.to_vec()).send()?;
    assert!(result.is_ok());

    assert_eq!(body, &result.raw_body()[..]);

    Ok(())
}
