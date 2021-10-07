// https://tools.ietf.org/id/draft-ietf-jose-json-web-key-00.html#rfc.section.3.1

use alcoholic_jwt::{token_kid, JWKS};
use chrono::{DateTime, Utc};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde::Serialize;

use crate::auth0::errors::Auth0Error;
use crate::auth0::token::Token;
use crate::auth0::Config;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonWebKeySet {
    keys: Vec<String>,
    expire_date: DateTime<Utc>,
}

impl JsonWebKeySet {
    pub fn is_signed(&self, token: Token) -> bool {
        let kid: String = token_kid(token.as_str())
            .unwrap_or_else(|| None)
            .unwrap_or_else(|| "-".to_string());

        self.keys.iter().find(|key| key.key_id() == kid).is_some()
    }

    pub fn keys(self) -> Vec<JsonWebKey> {
        self.keys
    }

    pub async fn fetch(client_ref: &Client, config_ref: &Config) -> Result<Self, Auth0Error> {
        let response: Response = client_ref
            .get(config_ref.jwks_url().clone())
            .send()
            .await
            .map_err(|e| {
                Auth0Error::JwksFetchError(config_ref.jwks_url().as_str().to_string(), e)
            })?;

        let jwks: JWKS = response.json::<JWKS>().await.map_err(|e| {
            Auth0Error::JwksFetchDeserializationError(config_ref.jwks_url().as_str().to_string(), e)
        })?;
    }
}
