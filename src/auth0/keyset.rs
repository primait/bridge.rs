// https://tools.ietf.org/id/draft-ietf-jose-json-web-key-00.html#rfc.section.3.1

use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

use crate::auth0::errors::Auth0Error;
use crate::auth0::token::Token;
use crate::auth0::Config;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonWebKeySet {
    keys: Vec<JsonWebKey>,
}

impl JsonWebKeySet {
    pub fn is_signed(&self, token_ref: &Token) -> bool {
        let kid: String = jsonwebtoken::decode_header(token_ref.as_str())
            .map(|headers| headers.kid)
            .unwrap_or(None)
            .unwrap_or_else(|| "-".to_string());

        self.keys.iter().any(|key| key.key_id() == kid)
    }

    pub async fn fetch(client_ref: &Client, config_ref: &Config) -> Result<Self, Auth0Error> {
        client_ref
            .get(config_ref.jwks_url().clone())
            .send()
            .await
            .map_err(|e| {
                Auth0Error::JwksFetchError(
                    e.status().map(|v| v.as_u16()).unwrap_or_default(),
                    config_ref.jwks_url().as_str().to_string(),
                    e,
                )
            })?
            .json::<Self>()
            .await
            .map_err(|e| {
                Auth0Error::JwksFetchDeserializationError(
                    config_ref.jwks_url().as_str().to_string(),
                    e,
                )
            })
    }
}

// https://tools.ietf.org/id/draft-ietf-jose-json-web-key-00.html#rfc.section.3
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kty")]
pub enum JsonWebKey {
    #[serde(alias = "RSA")]
    Rsa(RsaPublicJwk),
}

impl JsonWebKey {
    fn key_id(&self) -> &str {
        match self {
            JsonWebKey::Rsa(rsa_pk) => rsa_pk.key_id(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RsaPublicJwk {
    #[serde(rename = "kid")]
    key_id: String,
}

impl RsaPublicJwk {
    fn key_id(&self) -> &str {
        &self.key_id
    }
}
