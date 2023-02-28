use async_trait::async_trait;
use mockito::{Matcher, Mock};
use prima_bridge::{prelude::*, RedirectPolicy};
use reqwest::Url;

#[allow(unused)]
pub fn enable_mockito_logging() {
    std::env::set_var("RUST_LOG", "mockito=debug");
    env_logger::init();
}

#[async_trait]
pub trait MockitoServerCreateBridgeExt {
    async fn create_bridge(&mut self, status_code: usize, body: &str) -> (Mock, Bridge);
    async fn create_bridge_with_base_and_path(
        &mut self,
        status_code: usize,
        body: &str,
        base: &str,
        path: &str,
    ) -> (Mock, Bridge);
    async fn create_bridge_with_path(&mut self, status_code: usize, body: &str, path: &str) -> (Mock, Bridge);
    async fn create_bridge_with_path_and_header(
        &mut self,
        status_code: usize,
        body: &str,
        path: &str,
        header: (&str, &str),
    ) -> (Mock, Bridge);
    async fn create_bridge_with_raw_body_matcher(&mut self, body: &str) -> (Mock, Bridge);
    async fn create_bridge_with_header_matcher(&mut self, name_value: (&str, &str)) -> (Mock, Bridge);
    async fn create_bridge_with_user_agent(&mut self, user_agent: &str) -> (Mock, Bridge);
    async fn create_bridge_with_json_body_matcher(&mut self, json: serde_json::Value) -> (Mock, Bridge);
    async fn create_bridge_with_binary_body_matcher(&mut self, body: &[u8]) -> (Mock, Bridge);

    async fn create_bridge_with_redirect(
        &mut self,
        status_code: usize,
        body: &str,
        path: &str,
        redirect_to: &str,
        policy: RedirectPolicy,
    ) -> (Mock, Bridge);
}

#[async_trait]
impl MockitoServerCreateBridgeExt for mockito::Server {
    async fn create_bridge(&mut self, status_code: usize, body: &str) -> (Mock, Bridge) {
        self.create_bridge_with_path(status_code, body, "/").await
    }

    async fn create_bridge_with_base_and_path(
        &mut self,
        status_code: usize,
        body: &str,
        base: &str,
        path: &str,
    ) -> (Mock, Bridge) {
        let mock = self
            .mock("GET", format!("/{}/{}", base, path).as_str())
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .with_status(status_code)
            .with_body(body)
            .create_async()
            .await;

        let base_url = format!("{}/{}", self.url(), base);

        let url = Url::parse(base_url.as_str()).unwrap();
        let bridge = Bridge::builder().build(url);

        (mock, bridge)
    }

    async fn create_bridge_with_path(&mut self, status_code: usize, body: &str, path: &str) -> (Mock, Bridge) {
        let mock = self
            .mock("GET", path)
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .with_status(status_code)
            .with_body(body)
            .create_async()
            .await;

        let url = Url::parse(self.url().as_str()).unwrap();
        let bridge = Bridge::builder().build(url);

        (mock, bridge)
    }

    async fn create_bridge_with_path_and_header(
        &mut self,
        status_code: usize,
        body: &str,
        path: &str,
        header: (&str, &str),
    ) -> (Mock, Bridge) {
        let mock = self
            .mock("GET", path)
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .with_status(status_code)
            .with_header(header.0, header.1)
            .with_body(body)
            .create_async()
            .await;

        let url = Url::parse(self.url().as_str()).unwrap();
        let bridge = Bridge::builder().build(url);

        (mock, bridge)
    }

    async fn create_bridge_with_raw_body_matcher(&mut self, body: &str) -> (Mock, Bridge) {
        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_body(Matcher::Exact(body.to_owned()))
            .with_status(200)
            .create_async()
            .await;

        let url = Url::parse(self.url().as_str()).unwrap();
        let bridge = Bridge::builder().build(url);

        (mock, bridge)
    }

    async fn create_bridge_with_header_matcher(&mut self, (name, value): (&str, &str)) -> (Mock, Bridge) {
        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header(name, value)
            .with_status(200)
            .create_async()
            .await;

        let url = Url::parse(self.url().as_str()).unwrap();
        let bridge = Bridge::builder().build(url);

        (mock, bridge)
    }

    async fn create_bridge_with_user_agent(&mut self, user_agent: &str) -> (Mock, Bridge) {
        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header("User-Agent", user_agent)
            .with_status(200)
            .create_async()
            .await;

        let url = Url::parse(self.url().as_str()).unwrap();
        let bridge = Bridge::builder().with_user_agent(user_agent).build(url);

        (mock, bridge)
    }

    async fn create_bridge_with_json_body_matcher(&mut self, json: serde_json::Value) -> (Mock, Bridge) {
        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_body(Matcher::Json(json))
            .with_status(200)
            .create_async()
            .await;

        let url = Url::parse(self.url().as_str()).unwrap();
        let bridge = Bridge::builder().build(url);

        (mock, bridge)
    }

    async fn create_bridge_with_binary_body_matcher(&mut self, body: &[u8]) -> (Mock, Bridge) {
        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_body(Matcher::from(body.to_vec()))
            .with_status(200)
            .with_body(body)
            .create_async()
            .await;

        let url = Url::parse(self.url().as_str()).unwrap();
        let bridge = Bridge::builder().build(url);

        (mock, bridge)
    }

    async fn create_bridge_with_redirect(
        &mut self,
        status_code: usize,
        body: &str,
        path: &str,
        redirect_to: &str,
        policy: RedirectPolicy,
    ) -> (Mock, Bridge) {
        let mock = self
            .mock("GET", path)
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .with_status(status_code)
            .with_header("Location", redirect_to)
            .with_body(body)
            .create_async()
            .await;

        let url = Url::parse(self.url().as_str()).unwrap();
        let bridge = Bridge::builder().with_redirect_policy(policy).build(url);

        (mock, bridge)
    }
}
