use std::error::Error;
use std::time::Duration;

use mockito::Server;
use prima_bridge::auth0::cache::InMemoryCache;
use prima_bridge::auth0::Auth0Client;
use prima_bridge::auth0::RefreshingToken;
use prima_bridge::auth0::StalenessCheckPercentage;
use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::json;

use prima_bridge::prelude::*;

use crate::async_auth0::builder::*;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
struct Data {
    hello: String,
}

#[tokio::test]
async fn simple_request() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new_async().await;
    let (_m, bridge) = server.create_bridge(200, "{\"hello\": \"world!\"}").await;
    let result: String = RestRequest::new(&bridge).send().await?.get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[tokio::test]
async fn simple_request_with_auth0_scope() -> Result<(), Box<dyn Error>> {
    let mut auth0_server = Server::new_async().await;
    auth0_server.create_bridge(200, "{}").await;

    let mut server = Server::new_async().await;

    let client_id = "client_id";
    let client_secret = "client_secret";
    let audience = "audience";
    let scope = "profile email";

    let token_url = format!("{}/token", auth0_server.url().as_str()).parse().unwrap();

    let auth0_client = Auth0Client::new(
        token_url,
        reqwest::Client::default(),
        client_id.to_string(), 
        client_secret.to_string()
    );

    let token = RefreshingToken::new(
        auth0_client,
        Duration::from_secs(10),
        StalenessCheckPercentage::default(),
        Box::new(InMemoryCache::default()),
        audience.to_string(),
        Some(scope.to_string()),
    )
    .await
    .unwrap();

    let req_token_body = json!({
        "client_id": client_id,
        "client_secret": client_secret,
        "audience": audience,
        "grant_type": "client_credentials",
        "scope": scope,
    });

    let (_m, bridge) = server
        .create_bridge_with_auth0_get_token_match_body(
            json!({"hello": "world!"}),
            req_token_body,
            token,
        )
        .await;
    let result: String = RestRequest::new(&bridge).send().await?.get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}

#[tokio::test]
async fn unserializable_response() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new_async().await;
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
    let mut server = Server::new_async().await;
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

    let mut server = Server::new_async().await;
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
    let mut server = Server::new_async().await;
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
    let mut server = Server::new_async().await;
    let (_m, bridge) = server.create_bridge_with_raw_body_matcher("abcde").await;

    let result = RestRequest::new(&bridge).raw_body("abcde").send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn request_with_custom_json_body() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new_async().await;
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
    let mut server = Server::new_async().await;
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
    let mut server = Server::new_async().await;
    let (_m, bridge) = server.create_bridge_with_user_agent("test").await;

    let result = RestRequest::new(&bridge).send().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn request_with_binary_body_response() -> Result<(), Box<dyn Error>> {
    let body = b"abcde";

    let mut server = Server::new_async().await;
    let (_m, bridge) = server.create_bridge_with_binary_body_matcher(body).await;

    let result = RestRequest::new(&bridge).raw_body(body.to_vec()).send().await?;
    assert!(result.is_ok());

    assert_eq!(body, &result.raw_body()[..]);
    Ok(())
}

#[tokio::test]
async fn equal_headers_should_be_sent_only_once() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new_async().await;
    let (_m, bridge) = server.create_bridge(200, "{\"hello\": \"world!\"}").await;
    let req = RestRequest::new(&bridge).with_custom_headers(vec![
        (HeaderName::from_static("x-test"), HeaderValue::from_static("value")),
        (HeaderName::from_static("x-test"), HeaderValue::from_static("value")),
    ]);

    let headers = req.get_custom_headers();
    assert_eq!(HeaderValue::from_str("value").ok().as_ref(), headers.get("x-test"));

    Ok(())
}

#[cfg(feature = "gzip")]
#[tokio::test]
async fn decompresses_gzip_responses() -> Result<(), Box<dyn Error>> {
    use flate2::{write::GzEncoder, Compression};
    use std::io::prelude::*;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(b"{\"hello\": \"world!\"}")?;
    let body = encoder.finish()?;

    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("Content-Encoding", "gzip")
        .with_body(body)
        .create_async()
        .await;

    let _mocks = crate::async_auth0::Auth0Mocks::new(&mut server).await;
    let token = crate::async_auth0::refreshing_token(&server).await;

    let bridge = Bridge::builder()
        .with_refreshing_token(token)
        .await
        .build(server.url().parse().unwrap());

    let result: String = RestRequest::new(&bridge).send().await?.get_data(&["hello"])?;

    assert_eq!(result, "world!");
    Ok(())
}
