use mockito::{mock, BinaryBody, Matcher, Mock};
use prima_bridge::{prelude::*, RedirectPolicy};
use reqwest::Url;

#[allow(unused)]
pub fn enable_mockito_logging() {
    std::env::set_var("RUST_LOG", "mockito=debug");
    env_logger::init();
}

pub fn create_bridge(status_code: usize, body: &str) -> (Mock, Bridge) {
    create_bridge_with_path(status_code, body, "/")
}

pub fn create_bridge_with_base_and_path(status_code: usize, body: &str, base: &str, path: &str) -> (Mock, Bridge) {
    let mock = mock("GET", format!("/{}/{}", base, path).as_str())
        .match_header(
            "x-request-id",
            Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
        )
        .with_status(status_code)
        .with_body(body)
        .create();

    let base_url = format!("{}/{}", mockito::server_url(), base);

    let url = Url::parse(base_url.as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    (mock, bridge)
}

pub fn create_bridge_with_path(status_code: usize, body: &str, path: &str) -> (Mock, Bridge) {
    let mock = mock("GET", path)
        .match_header(
            "x-request-id",
            Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
        )
        .with_status(status_code)
        .with_body(body)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    (mock, bridge)
}

pub fn create_bridge_with_path_and_header(
    status_code: usize,
    body: &str,
    path: &str,
    header: (&str, &str),
) -> (Mock, Bridge) {
    let mock = mock("GET", path)
        .match_header(
            "x-request-id",
            Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
        )
        .with_status(status_code)
        .with_header(header.0, header.1)
        .with_body(body)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    (mock, bridge)
}

pub fn create_bridge_with_raw_body_matcher(body: &str) -> (Mock, Bridge) {
    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
        )
        .match_body(Matcher::Exact(body.to_owned()))
        .with_status(200)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    (mock, bridge)
}

pub fn create_bridge_with_header_matcher((name, value): (&str, &str)) -> (Mock, Bridge) {
    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
        )
        .match_header(name, value)
        .with_status(200)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    (mock, bridge)
}

pub fn create_bridge_with_user_agent(user_agent: &str) -> (Mock, Bridge) {
    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
        )
        .match_header("User-Agent", user_agent)
        .with_status(200)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().with_user_agent(user_agent).build(url);

    (mock, bridge)
}

pub fn create_bridge_with_json_body_matcher(json: serde_json::Value) -> (Mock, Bridge) {
    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
        )
        .match_body(Matcher::Json(json))
        .with_status(200)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    (mock, bridge)
}

pub fn create_bridge_with_binary_body_matcher(body: &[u8]) -> (Mock, Bridge) {
    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
        )
        .match_body(Matcher::Binary(BinaryBody::from_bytes(body.to_vec())))
        .with_status(200)
        .with_body(body)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    (mock, bridge)
}

pub fn create_bridge_with_redirect(
    status_code: usize,
    body: &str,
    path: &str,
    redirect_to: &str,
    policy: RedirectPolicy,
) -> (Mock, Bridge) {
    let mock = mock("GET", path)
        .match_header(
            "x-request-id",
            Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
        )
        .with_status(status_code)
        .with_header("Location", redirect_to)
        .with_body(body)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().with_redirect_policy(policy).build(url);

    (mock, bridge)
}
