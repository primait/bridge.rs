use std::{collections::HashSet, error::Error};

use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::json;

use prima_bridge::{prelude::*, MultipartFile, MultipartFormFileField, RedirectPolicy, RestMultipart};

use crate::common::*;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
struct Data {
    hello: String,
}

#[tokio::test]
async fn simple_request() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge(200, "{\"hello\": \"world!\"}").await;
    let result: String = RestRequest::new(&bridge).send().await?.get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[tokio::test]
async fn request_with_redirect_policy_none() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;

    let _destination_mock = server
        .mock("GET", "/destination")
        .with_status(200)
        .with_body("{\"after\": \"redirect\"}")
        .create_async()
        .await;

    let redirect_to = format!("{}/destination", server.url());
    let body = "{\"before\": \"redirect\"}";
    let (_m, bridge) = server
        .create_bridge_with_redirect(302, body, "/", &redirect_to, RedirectPolicy::NoFollow)
        .await;

    let result: Response = RestRequest::new(&bridge)
        .ignore_status_code()
        .send()
        .await
        .expect("request failed");

    assert!(result.status_code().is_redirection());
    assert_eq!(result.headers().get("Location").unwrap().to_str().unwrap(), redirect_to);
    let response: serde_json::Value =
        serde_json::from_slice(result.raw_body()).expect("Failed to deserialize response");
    assert_eq!(response, json!({"before": "redirect"}));

    Ok(())
}

#[tokio::test]
async fn request_with_redirect_policy_follow() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;

    let _destination_mock = server
        .mock("GET", "/destination")
        .with_status(200)
        .with_body("{\"after\": \"redirect\"}")
        .create_async()
        .await;

    let redirect_to = format!("{}/destination", server.url());
    let body = "{\"before\": \"redirect\"}";
    let (_m, bridge) = server
        .create_bridge_with_redirect(302, body, "/", &redirect_to, RedirectPolicy::Limited(2))
        .await;

    let result: Response = RestRequest::new(&bridge)
        .ignore_status_code()
        .send()
        .await
        .expect("request failed");

    assert!(result.status_code().is_success());
    let response: serde_json::Value =
        serde_json::from_slice(result.raw_body()).expect("Failed to deserialize response");
    assert_eq!(response, json!({"after": "redirect"}));

    Ok(())
}

#[tokio::test]
async fn unserializable_response() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge(200, "{\"hello\": \"world!\"}").await;

    let result: PrimaBridgeResult<Response> = RestRequest::new(&bridge).send().await;
    assert!(result.is_ok());
    let result: PrimaBridgeResult<Data> = result?.get_data(&["some_strange_selector"]);
    assert!(result.is_err());
    let error_str = result.err().map(|e| e.to_string()).unwrap();
    assert!(error_str.contains("the data for key `some_strange_selector`"));

    Ok(())
}

#[tokio::test]
async fn simple_request_with_custom_path_and_base_path() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server
        .create_bridge_with_base_and_path(200, "{\"hello\": \"world!\"}", "api", "test_path")
        .await;

    let result: String = RestRequest::new(&bridge)
        .to("test_path")
        .send()
        .await?
        .get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[tokio::test]
async fn simple_request_with_custom_headers() -> Result<(), Box<dyn Error>> {
    let header = ("header", "custom");
    let path = "/test_path/simple_request_with_custom_headers";

    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server
        .create_bridge_with_path_and_header(200, "{\"hello\": \"world!\"}", path, header)
        .await;

    let response = RestRequest::new(&bridge).to(path).send().await?;

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

#[tokio::test]
async fn simple_request_with_wrong_status_code() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge(403, "{\"hello\": \"world!\"}").await;

    let result: String = RestRequest::new(&bridge)
        .ignore_status_code()
        .send()
        .await?
        .get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[tokio::test]
async fn request_with_custom_body() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge_with_raw_body_matcher("abcde").await;

    let result = RestRequest::new(&bridge).raw_body("abcde").send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn request_with_custom_json_body() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server
        .create_bridge_with_json_body_matcher(json!({"hello": "world"}))
        .await;
    let data = Data {
        hello: "world".to_string(),
    };
    let result = RestRequest::new(&bridge).json_body(&data)?.send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn request_with_custom_headers() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server
        .create_bridge_with_header_matcher(("x-prima", "test-value"))
        .await;

    let result = RestRequest::new(&bridge)
        .with_custom_headers(vec![(
            HeaderName::from_static("x-prima"),
            HeaderValue::from_static("test-value"),
        )])
        .send()
        .await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn request_with_custom_user_agent() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge_with_user_agent("test").await;

    let result = RestRequest::new(&bridge).send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn request_with_binary_body_response() -> Result<(), Box<dyn Error>> {
    let body = b"abcde";

    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge_with_binary_body_matcher(body).await;

    let result = RestRequest::new(&bridge).raw_body(body.to_vec()).send().await?;
    assert!(result.is_ok());

    assert_eq!(body, &result.raw_body()[..]);
    Ok(())
}

#[tokio::test]
async fn equal_headers_should_be_sent_only_once() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge(200, "{\"hello\": \"world!\"}").await;

    let req = RestRequest::new(&bridge).with_custom_headers(vec![
        (HeaderName::from_static("x-test"), HeaderValue::from_static("value")),
        (HeaderName::from_static("x-test"), HeaderValue::from_static("value")),
    ]);

    let headers = req.get_custom_headers();
    assert_eq!(HeaderValue::from_str("value").ok().as_ref(), headers.get("x-test"));

    Ok(())
}

#[tokio::test]
async fn get_body_returns_serialized_json_body() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server
        .create_bridge_with_json_body_matcher(json!({"hello": "world"}))
        .await;

    let data = Data {
        hello: "world".to_string(),
    };
    let data_json = serde_json::to_string(&data)?;

    let request = RestRequest::new(&bridge).json_body(&data)?;
    assert_eq!(request.get_body(), Some(data_json.as_bytes()));

    let result = request.send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn get_body_returns_raw_body() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge(200, "{\"hello\": \"world!\"}").await;

    let data = b"Hello, world!".as_slice();
    let request = RestRequest::new(&bridge).raw_body(data);
    assert_eq!(request.get_body(), Some(data));

    let result = request.send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn get_body_returns_none_when_body_is_stream() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge(200, "{\"hello\": \"world!\"}").await;

    let file = tokio::fs::File::open("tests/resources/howdy_world.txt").await?;
    let request = RestRequest::new(&bridge).raw_body(file);
    assert_eq!(request.get_body(), None);

    let result = request.send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn get_body_returns_none_when_request_is_multipart() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge(200, "{\"hello\": \"world!\"}").await;

    let data = RestMultipart::multiple(HashSet::from_iter([MultipartFormFileField::new(
        "file0",
        MultipartFile::new("Hello, world!")
            .with_name("hello_world.txt")
            .with_mime_type("text/plain"),
    )]));

    let request = RestRequest::new(&bridge).multipart_body(data);
    assert_eq!(request.get_body(), None);

    let result = request.send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn get_body_returns_none_when_request_has_no_body() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new_async().await;
    let (_m, bridge) = server.create_bridge(200, "{\"hello\": \"world!\"}").await;

    let request = RestRequest::new(&bridge);
    assert_eq!(request.get_body(), None);

    let result = request.send().await;
    assert!(result.is_ok());
    Ok(())
}

#[cfg(feature = "gzip")]
#[tokio::test]
async fn decompresses_gzip_responses() {
    use flate2::{write::GzEncoder, Compression};
    use std::io::prelude::*;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(b"{\"hello\": \"world!\"}").unwrap();
    let body = encoder.finish().unwrap();

    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("Content-Encoding", "gzip")
        .with_body(body)
        .create_async()
        .await;

    let bridge = Bridge::builder().build(server.url().parse().unwrap());

    let result: String = RestRequest::new(&bridge)
        .send()
        .await
        .unwrap()
        .get_data(&["hello"])
        .unwrap();
    assert_eq!(result, "world!");
}
