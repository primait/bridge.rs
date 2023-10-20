use std::str::FromStr;
use std::time::Duration;

use jsonwebtoken::Algorithm;
use mockito::{Matcher, Mock, Server};
use reqwest::Url;

use prima_bridge::auth0::{CacheType, Config, StalenessCheckPercentage};

mod builder;
mod graphql;
mod rest;

fn config(server: &Server) -> Config {
    Config {
        token_url: Url::from_str(&format!("{}/{}", server.url().as_str(), "token")).unwrap(),
        jwks_url: Url::from_str(&format!("{}/{}", server.url().as_str(), "jwks")).unwrap(),
        caller: "caller".to_string(),
        audience: "audience".to_string(),
        cache_type: CacheType::Inmemory,
        token_encryption_key: "32char_long_token_encryption_key".to_string(),
        check_interval: Duration::from_secs(10),
        staleness_check_percentage: StalenessCheckPercentage::default(),
        client_id: "client_id".to_string(),
        client_secret: "client_secret".to_string(),
        scope: None,
    }
}

struct Auth0Mocks {
    _jwks: Mock,
    _token: Mock,
    endpoint: Option<Mock>,
}

impl Auth0Mocks {
    pub async fn new(server: &mut mockito::Server) -> Self {
        let jwks: Mock = Auth0Mocks::get_jwks_mock(server).await;
        let token: Mock = Auth0Mocks::get_token_mock(server, None).await;

        Self {
            _jwks: jwks,
            _token: token,
            endpoint: None,
        }
    }

    pub async fn new_with_req_token_body_match(server: &mut mockito::Server, match_body: serde_json::Value) -> Self {
        let jwks: Mock = Auth0Mocks::get_jwks_mock(server).await;
        let token: Mock = Auth0Mocks::get_token_mock(server, Some(match_body)).await;

        Self {
            _jwks: jwks,
            _token: token,
            endpoint: None,
        }
    }

    async fn get_jwks_mock(server: &mut mockito::Server) -> Mock {
        let content: String = std::fs::read_to_string("tests/resources/auth0/jwks.json").unwrap();
        server
            .mock("GET", "/jwks")
            .with_status(200)
            .with_body(content)
            .create_async()
            .await
    }

    async fn get_token_mock(server: &mut mockito::Server, match_body: Option<serde_json::Value>) -> Mock {
        let claims: Claims = Claims::new("test".to_string());
        let object: FetchTokenResponse = FetchTokenResponse {
            access_token: claims.as_jwt(),
            scope: "ciao".to_string(),
            expires_in: 86400,
            token_type: "ciao".to_string(),
        };
        let mut mock = server
            .mock("POST", "/token")
            .match_header("content-type", "application/json");

        if let Some(body) = match_body {
            mock = mock.match_body(Matcher::Json(body))
        }

        mock = mock
            .with_status(200)
            .with_body(serde_json::to_string(&object).unwrap())
            .create_async()
            .await;
        mock
    }

    pub fn set_endpoint_mock(&mut self, endpoint: Mock) -> &Self {
        self.endpoint = Some(endpoint);
        self
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Claims {
    aud: String,
    exp: Option<i64>,
}

impl Claims {
    pub fn new(aud: String) -> Self {
        Self { aud, exp: Some(86400) }
    }

    pub fn as_jwt(&self) -> String {
        let content: String = std::fs::read_to_string("tests/resources/auth0/test_cert_key.pem").unwrap();
        jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS256),
            &self,
            &jsonwebtoken::EncodingKey::from_rsa_pem(content.as_ref()).unwrap(),
        )
        .unwrap()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchTokenResponse {
    access_token: String,
    scope: String,
    expires_in: i32,
    token_type: String,
}
