use crate::common::auth0_context;
use mockito::{mock, Mock};
use prima_bridge::auth0_config::Auth0Config;
use reqwest::header::AUTHORIZATION;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub scope: String,
    pub expires_in: i32,
    pub token_type: String,
}

pub struct Auth0Context {
    auth0_server: Mock,
    generic_service: Mock,
}

impl Auth0Context {
    pub fn default_config(&self) -> Auth0Config {
        Auth0Config::new(
            reqwest::Url::parse(&format!("{}/{}", mockito::server_url().as_str(), "token"))
                .unwrap(),
            reqwest::Url::parse(&format!("{}/{}", mockito::server_url().as_str(), "jwks")).unwrap(),
            "caller".to_string(),
            "audience".to_string(),
            "none".to_string(),
            "32char_long_token_encryption_key".to_string(),
            std::time::Duration::from_secs(1),
            1,
            60,
            "client_id".to_string(),
            "client_secret".to_string(),
        )
    }

    pub fn new(token: impl ToString) -> Self {
        let auth0_server = Self::mock_auth0_server(&token);
        let generic_service = Self::mock_generic_service(&token);

        Self {
            auth0_server,
            generic_service,
        }
    }

    pub fn change_token_on_service(&mut self, new_token: impl ToString) {
        self.generic_service = Self::mock_generic_service(&new_token);
    }

    pub fn change_token_on_auth0(&mut self, new_token: impl ToString) {
        self.auth0_server = Self::mock_auth0_server(&new_token);
    }

    fn mock_generic_service(token: &impl ToString) -> Mock {
        mock("GET", "/api")
            .match_header(
                AUTHORIZATION.as_str(),
                format!("Bearer {}", token.to_string()).as_str(),
            )
            .with_status(200)
            .with_body("OK")
            .create()
    }

    fn mock_auth0_server(token: &impl ToString) -> Mock {
        let token_response = auth0_context::TokenResponse {
            access_token: token.to_string(),
            scope: "test".to_string(),
            expires_in: 10,
            token_type: "test".to_string(),
        };
        mock("POST", "/token")
            .with_status(200)
            .with_body(serde_json::to_string(&token_response).unwrap())
            .create()
    }
}
