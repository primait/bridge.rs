use bridge::prelude::*;
use mockito::{mock, Mock};
use reqwest::Url;

pub fn create_bridge(status_code: usize, body: &str) -> (Mock, Bridge) {
    create_bridge_with_path(status_code, body, "/")
}

pub fn create_bridge_with_base_and_path(
    status_code: usize,
    body: &str,
    base: &str,
    path: &str,
) -> (Mock, Bridge) {
    let mock = mock("GET", format!("/{}/{}", base, path).as_str())
        .with_status(status_code)
        .with_body(body)
        .create();

    let base_url = format!("{}/{}", mockito::server_url(), base);

    let url = Url::parse(base_url.as_str()).unwrap();
    let bridge = Bridge::new(url);

    (mock, bridge)
}

pub fn create_bridge_with_path(status_code: usize, body: &str, path: &str) -> (Mock, Bridge) {
    let mock = mock("GET", path)
        .with_status(status_code)
        .with_body(body)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::new(url);

    (mock, bridge)
}
