use std::error::Error;

use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::json;

use prima_bridge::prelude::*;

use crate::common::*;
use crate::Generator;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
struct Data {
    hello: String,
}

#[tokio::test]
async fn simple_request() -> Result<(), Box<dyn Error>> {
    let (_m, url) = get_mock(200, "{\"hello\": \"world!\"}");
    let bridge: Bridge = Generator::bridge(url).await;
    let result: String = RestRequest::new(&bridge)
        .send()
        .await?
        .get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[tokio::test]
async fn unserializable_response() -> Result<(), Box<dyn Error>> {
    let (_m, url) = get_mock(200, "{\"hello\": \"world!\"}");
    let bridge: Bridge = Generator::bridge(url).await;
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
    let (_m, url) = mock_with_base_and_path(200, "{\"hello\": \"world!\"}", "api", "test_path");
    let bridge: Bridge = Generator::bridge(url).await;
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

    let (_m, url) = mock_with_path_and_header(200, "{\"hello\": \"world!\"}", path, header);
    let bridge: Bridge = Generator::bridge(url).await;
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
    let (_m, url) = get_mock(403, "{\"hello\": \"world!\"}");
    let bridge: Bridge = Generator::bridge(url).await;
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
    let (_m, url) = mock_with_raw_body_matcher("abcde");
    let bridge: Bridge = Generator::bridge(url).await;
    let result = RestRequest::new(&bridge).raw_body("abcde").send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn request_with_custom_json_body() -> Result<(), Box<dyn Error>> {
    let (_m, url) = mock_with_json_body_matcher(json!({"hello": "world"}));
    let bridge: Bridge = Generator::bridge(url).await;
    let data = Data {
        hello: "world".to_string(),
    };
    let result = RestRequest::new(&bridge).json_body(&data)?.send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn request_with_custom_headers() -> Result<(), Box<dyn Error>> {
    let (_m, url) = mock_with_header_matcher(("x-prima", "test-value"));
    let bridge: Bridge = Generator::bridge(url).await;
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
    let (_m, url) = mock_with_user_agent("test");
    let bridge: Bridge = Generator::bridge_with_user_agent(url, "test").await;
    let result = RestRequest::new(&bridge).send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn request_with_binary_body_response() -> Result<(), Box<dyn Error>> {
    let body = b"abcde";

    let (_m, url) = mock_with_binary_body_matcher(body);
    let bridge: Bridge = Generator::bridge(url).await;
    let result = RestRequest::new(&bridge)
        .raw_body(body.to_vec())
        .send()
        .await?;
    assert!(result.is_ok());

    assert_eq!(body, &result.raw_body()[..]);
    Ok(())
}

#[tokio::test]
async fn equal_headers_should_be_sent_only_once() -> Result<(), Box<dyn Error>> {
    let (_m, url) = get_mock(200, "{\"hello\": \"world!\"}");
    let bridge: Bridge = Generator::bridge(url).await;
    let req = RestRequest::new(&bridge).with_custom_headers(vec![
        (
            HeaderName::from_static("x-test"),
            HeaderValue::from_static("value"),
        ),
        (
            HeaderName::from_static("x-test"),
            HeaderValue::from_static("value"),
        ),
    ]);

    let headers = req.get_custom_headers();
    assert_eq!(
        HeaderValue::from_str("value").ok().as_ref(),
        headers.get("x-test")
    );

    Ok(())
}

#[cfg(feature = "gzip")]
#[tokio::test]
async fn gzip_compression() -> Result<(), Box<dyn Error>> {
    use flate2::{write::GzEncoder, Compression};
    use std::io::prelude::*;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write(b"{\"hello\": \"world!\"}")?;
    let body = encoder.finish()?;
    let _mock = mockito::mock("GET", "/")
        .with_status(200)
        .with_header("Content-Encoding", "gzip")
        .with_body(body)
        .create();

    let bridge = Bridge::new(mockito::server_url().parse().unwrap());

    let result: String = RestRequest::new(&bridge)
        .send()
        .await?
        .get_data(&["hello"])?;
    assert_eq!(result, "world!");

    Ok(())
}

#[cfg(feature = "auth0")]
#[tokio::test]
async fn request_with_auth0() -> Result<(), Box<dyn Error>> {
    let body = b"abcde";
    let (_m, url) = mock_with_binary_body_matcher(body);
    let _auth0 = create_auth0_mock();

    let config = prima_bridge::auth0_config::Auth0Config::new(
        reqwest::Url::parse(&format!("{}/{}", mockito::server_url().as_str(), "token")).unwrap(),
        reqwest::Url::parse(&format!("{}/{}", mockito::server_url().as_str(), "jwks")).unwrap(),
        "caller".to_string(),
        "audience".to_string(),
        "none".to_string(),
        "caller".to_string(),
        "32char_long_token_encryption_key".to_string(),
        std::time::Duration::from_secs(10),
        1,
        60,
        "client_id".to_string(),
        "client_secret".to_string(),
    );

    let bridge: Bridge = Generator::bridge_with_auth0_config(url, config).await;

    let req = RestRequest::new(&bridge);
    let mut h = reqwest::header::HeaderMap::new();
    h.insert("x-token", HeaderValue::from_static("abcdef"));
    assert_eq!(h, req.get_bridge().get_headers().await);

    Ok(())
}
