use async_trait::async_trait;
use mockito::{Matcher, Server};
use reqwest::Url;

use crate::async_auth0::Auth0Mocks;
use prima_bridge::auth0::RefreshingToken;
use prima_bridge::prelude::*;

use super::refreshing_token;

#[async_trait]
pub(in crate::async_auth0) trait Auth0MocksExt {
    async fn create_bridge(&mut self, status_code: usize, body: &str) -> (Auth0Mocks, Bridge);
    async fn create_bridge_with_base_and_path(
        &mut self,
        status_code: usize,
        body: &str,
        base: &str,
        path: &str,
    ) -> (Auth0Mocks, Bridge);

    async fn create_bridge_with_path(&mut self, status_code: usize, body: &str, path: &str) -> (Auth0Mocks, Bridge);

    async fn create_bridge_with_path_and_header(
        &mut self,
        status_code: usize,
        body: &str,
        path: &str,
        header: (&str, &str),
    ) -> (Auth0Mocks, Bridge);
    async fn create_bridge_with_raw_body_matcher(&mut self, body: &str) -> (Auth0Mocks, Bridge);

    async fn create_bridge_with_header_matcher(&mut self, (name, value): (&str, &str)) -> (Auth0Mocks, Bridge);
    async fn create_bridge_with_user_agent(&mut self, user_agent: &str) -> (Auth0Mocks, Bridge);
    async fn create_bridge_with_json_body_matcher(&mut self, json: serde_json::Value) -> (Auth0Mocks, Bridge);
    async fn create_bridge_with_binary_body_matcher(&mut self, body: &[u8]) -> (Auth0Mocks, Bridge);
    async fn create_bridge_with_auth0_get_token_match_body(
        &mut self,
        body: serde_json::Value,
        req_token_body: serde_json::Value,
        token: RefreshingToken,
    ) -> (Auth0Mocks, Bridge);
}

#[async_trait]
impl Auth0MocksExt for Server {
    async fn create_bridge(&mut self, status_code: usize, body: &str) -> (Auth0Mocks, Bridge) {
        self.create_bridge_with_path(status_code, body, "/").await
    }

    async fn create_bridge_with_base_and_path(
        &mut self,
        status_code: usize,
        body: &str,
        base: &str,
        path: &str,
    ) -> (Auth0Mocks, Bridge) {
        let base_url = format!("{}/{}", self.url(), base);
        let url = Url::parse(base_url.as_str()).unwrap();
        let mut mocks: Auth0Mocks = Auth0Mocks::new(self).await;
        let bridge = Bridge::builder()
            .with_refreshing_token(refreshing_token(self).await)
            .await
            .build(url);

        let mock = self
            .mock("GET", format!("/{base}/{path}").as_str())
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header(
                reqwest::header::AUTHORIZATION.as_str(),
                bridge.token().unwrap().to_bearer().as_str(),
            )
            .with_status(status_code)
            .with_body(body)
            .create_async()
            .await;

        mocks.set_endpoint_mock(mock);

        (mocks, bridge)
    }

    async fn create_bridge_with_auth0_get_token_match_body(
        &mut self,
        body: serde_json::Value,
        req_token_body: serde_json::Value,
        token: RefreshingToken,
    ) -> (Auth0Mocks, Bridge) {
        let url = Url::parse(self.url().as_str()).unwrap();
        let mut mocks: Auth0Mocks = Auth0Mocks::new_with_req_token_body_match(self, req_token_body).await;
        let bridge = Bridge::builder().with_refreshing_token(token).await.build(url);

        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header(
                reqwest::header::AUTHORIZATION.as_str(),
                bridge.token().unwrap().to_bearer().as_str(),
            )
            .with_status(200)
            .with_body(body.to_string())
            .create_async()
            .await;

        mocks.set_endpoint_mock(mock);

        (mocks, bridge)
    }

    async fn create_bridge_with_path(&mut self, status_code: usize, body: &str, path: &str) -> (Auth0Mocks, Bridge) {
        let url = Url::parse(self.url().as_str()).unwrap();
        let mut mocks: Auth0Mocks = Auth0Mocks::new(self).await;
        let bridge = Bridge::builder()
            .with_refreshing_token(refreshing_token(self).await)
            .await
            .build(url);

        let mock = self
            .mock("GET", path)
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header(
                reqwest::header::AUTHORIZATION.as_str(),
                bridge.token().unwrap().to_bearer().as_str(),
            )
            .with_status(status_code)
            .with_body(body)
            .create_async()
            .await;

        mocks.set_endpoint_mock(mock);

        (mocks, bridge)
    }

    async fn create_bridge_with_path_and_header(
        &mut self,
        status_code: usize,
        body: &str,
        path: &str,
        header: (&str, &str),
    ) -> (Auth0Mocks, Bridge) {
        let url = Url::parse(self.url().as_str()).unwrap();
        let mut mocks: Auth0Mocks = Auth0Mocks::new(self).await;
        let bridge = Bridge::builder()
            .with_refreshing_token(refreshing_token(self).await)
            .await
            .build(url);

        let mock = self
            .mock("GET", path)
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header(
                reqwest::header::AUTHORIZATION.as_str(),
                bridge.token().unwrap().to_bearer().as_str(),
            )
            .with_status(status_code)
            .with_header(header.0, header.1)
            .with_body(body)
            .create_async()
            .await;

        mocks.set_endpoint_mock(mock);

        (mocks, bridge)
    }

    async fn create_bridge_with_raw_body_matcher(&mut self, body: &str) -> (Auth0Mocks, Bridge) {
        let url = Url::parse(self.url().as_str()).unwrap();
        let mut mocks: Auth0Mocks = Auth0Mocks::new(self).await;
        let bridge = Bridge::builder()
            .with_refreshing_token(refreshing_token(self).await)
            .await
            .build(url);

        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header(
                reqwest::header::AUTHORIZATION.as_str(),
                bridge.token().unwrap().to_bearer().as_str(),
            )
            .match_body(Matcher::Exact(body.to_owned()))
            .with_status(200)
            .create_async()
            .await;

        mocks.set_endpoint_mock(mock);

        (mocks, bridge)
    }

    async fn create_bridge_with_header_matcher(&mut self, (name, value): (&str, &str)) -> (Auth0Mocks, Bridge) {
        let url = Url::parse(self.url().as_str()).unwrap();
        let mut mocks: Auth0Mocks = Auth0Mocks::new(self).await;
        let bridge = Bridge::builder()
            .with_refreshing_token(refreshing_token(self).await)
            .await
            .build(url);

        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header(
                reqwest::header::AUTHORIZATION.as_str(),
                bridge.token().unwrap().to_bearer().as_str(),
            )
            .match_header(name, value)
            .with_status(200)
            .create_async()
            .await;

        mocks.set_endpoint_mock(mock);

        (mocks, bridge)
    }

    async fn create_bridge_with_user_agent(&mut self, user_agent: &str) -> (Auth0Mocks, Bridge) {
        let url = Url::parse(self.url().as_str()).unwrap();
        let mut mocks: Auth0Mocks = Auth0Mocks::new(self).await;
        let bridge = Bridge::builder()
            .with_refreshing_token(refreshing_token(self).await)
            .await
            .with_user_agent(user_agent)
            .build(url);

        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header(
                reqwest::header::AUTHORIZATION.as_str(),
                bridge.token().unwrap().to_bearer().as_str(),
            )
            .match_header("User-Agent", user_agent)
            .with_status(200)
            .create_async()
            .await;

        mocks.set_endpoint_mock(mock);

        (mocks, bridge)
    }

    async fn create_bridge_with_json_body_matcher(&mut self, json: serde_json::Value) -> (Auth0Mocks, Bridge) {
        let url = Url::parse(self.url().as_str()).unwrap();
        let mut mocks: Auth0Mocks = Auth0Mocks::new(self).await;
        let bridge = Bridge::builder()
            .with_refreshing_token(refreshing_token(self).await)
            .await
            .build(url);

        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header(
                reqwest::header::AUTHORIZATION.as_str(),
                bridge.token().unwrap().to_bearer().as_str(),
            )
            .match_body(Matcher::Json(json))
            .with_status(200)
            .create_async()
            .await;

        mocks.set_endpoint_mock(mock);

        (mocks, bridge)
    }

    async fn create_bridge_with_binary_body_matcher(&mut self, body: &[u8]) -> (Auth0Mocks, Bridge) {
        let url = Url::parse(self.url().as_str()).unwrap();
        let mut mocks: Auth0Mocks = Auth0Mocks::new(self).await;
        let bridge = Bridge::builder()
            .with_refreshing_token(refreshing_token(self).await)
            .await
            .build(url);

        let mock = self
            .mock("GET", "/")
            .match_header(
                "x-request-id",
                Matcher::Regex(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string()),
            )
            .match_header(
                reqwest::header::AUTHORIZATION.as_str(),
                bridge.token().unwrap().to_bearer().as_str(),
            )
            .match_body(Matcher::from(body.to_vec()))
            .with_status(200)
            .with_body(body)
            .create_async()
            .await;

        mocks.set_endpoint_mock(mock);

        (mocks, bridge)
    }
}
