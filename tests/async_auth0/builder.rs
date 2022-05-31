use mockito::{mock, BinaryBody, Matcher};
use reqwest::Url;

use prima_bridge::prelude::*;

use crate::async_auth0::{config, Auth0Mocks};

pub(in crate::async_auth0) async fn create_bridge(
    status_code: usize,
    body: &str,
) -> (Auth0Mocks, Bridge) {
    create_bridge_with_path(status_code, body, "/").await
}

pub(in crate::async_auth0) async fn create_bridge_with_base_and_path(
    status_code: usize,
    body: &str,
    base: &str,
    path: &str,
) -> (Auth0Mocks, Bridge) {
    let base_url = format!("{}/{}", mockito::server_url(), base);
    let url = Url::parse(base_url.as_str()).unwrap();
    let mut mocks: Auth0Mocks = Auth0Mocks::new();
    let bridge = Bridge::builder().with_auth0(config()).await.build(url);

    let mock = mock("GET", format!("/{}/{}", base, path).as_str())
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_header(
            reqwest::header::AUTHORIZATION.as_str(),
            bridge.token().unwrap().to_bearer().as_str(),
        )
        .with_status(status_code)
        .with_body(body)
        .create();

    mocks.set_endpoint_mock(mock);

    (mocks, bridge)
}

pub(in crate::async_auth0) async fn create_bridge_with_path(
    status_code: usize,
    body: &str,
    path: &str,
) -> (Auth0Mocks, Bridge) {
    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let mut mocks: Auth0Mocks = Auth0Mocks::new();
    let bridge = Bridge::builder().with_auth0(config()).await.build(url);

    let mock = mock("GET", path)
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_header(
            reqwest::header::AUTHORIZATION.as_str(),
            bridge.token().unwrap().to_bearer().as_str(),
        )
        .with_status(status_code)
        .with_body(body)
        .create();

    mocks.set_endpoint_mock(mock);

    (mocks, bridge)
}

pub(in crate::async_auth0) async fn create_bridge_with_path_and_header(
    status_code: usize,
    body: &str,
    path: &str,
    header: (&str, &str),
) -> (Auth0Mocks, Bridge) {
    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let mut mocks: Auth0Mocks = Auth0Mocks::new();
    let bridge = Bridge::builder().with_auth0(config()).await.build(url);

    let mock = mock("GET", path)
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_header(
            reqwest::header::AUTHORIZATION.as_str(),
            bridge.token().unwrap().to_bearer().as_str(),
        )
        .with_status(status_code)
        .with_header(header.0, header.1)
        .with_body(body)
        .create();

    mocks.set_endpoint_mock(mock);

    (mocks, bridge)
}

pub(in crate::async_auth0) async fn create_bridge_with_raw_body_matcher(
    body: &str,
) -> (Auth0Mocks, Bridge) {
    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let mut mocks: Auth0Mocks = Auth0Mocks::new();
    let bridge = Bridge::builder().with_auth0(config()).await.build(url);

    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_header(
            reqwest::header::AUTHORIZATION.as_str(),
            bridge.token().unwrap().to_bearer().as_str(),
        )
        .match_body(Matcher::Exact(body.to_owned()))
        .with_status(200)
        .create();

    mocks.set_endpoint_mock(mock);

    (mocks, bridge)
}

pub(in crate::async_auth0) async fn create_bridge_with_header_matcher(
    (name, value): (&str, &str),
) -> (Auth0Mocks, Bridge) {
    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let mut mocks: Auth0Mocks = Auth0Mocks::new();
    let bridge = Bridge::builder().with_auth0(config()).await.build(url);

    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_header(
            reqwest::header::AUTHORIZATION.as_str(),
            bridge.token().unwrap().to_bearer().as_str(),
        )
        .match_header(name, value)
        .with_status(200)
        .create();

    mocks.set_endpoint_mock(mock);

    (mocks, bridge)
}

pub(in crate::async_auth0) async fn create_bridge_with_user_agent(
    user_agent: &str,
) -> (Auth0Mocks, Bridge) {
    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let mut mocks: Auth0Mocks = Auth0Mocks::new();
    let bridge = Bridge::builder()
        .with_auth0(config())
        .await
        .with_user_agent(user_agent)
        .build(url);

    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_header(
            reqwest::header::AUTHORIZATION.as_str(),
            bridge.token().unwrap().to_bearer().as_str(),
        )
        .match_header("User-Agent", user_agent)
        .with_status(200)
        .create();

    mocks.set_endpoint_mock(mock);

    (mocks, bridge)
}

pub(in crate::async_auth0) async fn create_bridge_with_json_body_matcher(
    json: serde_json::Value,
) -> (Auth0Mocks, Bridge) {
    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let mut mocks: Auth0Mocks = Auth0Mocks::new();
    let bridge = Bridge::builder().with_auth0(config()).await.build(url);

    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_header(
            reqwest::header::AUTHORIZATION.as_str(),
            bridge.token().unwrap().to_bearer().as_str(),
        )
        .match_body(Matcher::Json(json))
        .with_status(200)
        .create();

    mocks.set_endpoint_mock(mock);

    (mocks, bridge)
}

pub(in crate::async_auth0) async fn create_bridge_with_binary_body_matcher(
    body: &[u8],
) -> (Auth0Mocks, Bridge) {
    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let mut mocks: Auth0Mocks = Auth0Mocks::new();
    let bridge = Bridge::builder().with_auth0(config()).await.build(url);

    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_header(
            reqwest::header::AUTHORIZATION.as_str(),
            bridge.token().unwrap().to_bearer().as_str(),
        )
        .match_body(Matcher::Binary(BinaryBody::from_bytes(body.to_vec())))
        .with_status(200)
        .with_body(body)
        .create();

    mocks.set_endpoint_mock(mock);

    (mocks, bridge)
}
