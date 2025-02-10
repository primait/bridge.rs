use crate::auth0::Auth0Error;
use chrono::{DateTime, Utc};
use reqwest::{Client, Response, Url};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{Config, Token};

/// The successful response received from the authorization server containing the access token.
/// Related [RFC](https://www.rfc-editor.org/rfc/rfc6749#section-5.1)
#[derive(Deserialize, Serialize, Debug)]
struct FetchTokenResponse {
    access_token: String,
    scope: Option<String>,
    expires_in: i32,
    token_type: String,
}

#[derive(Serialize, Debug)]
struct FetchTokenRequest {
    client_id: String,
    client_secret: String,
    audience: String,
    grant_type: String,
    scope: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Claims {
    #[allow(dead_code)]
    #[serde(default)]
    pub permissions: Vec<String>,
}

// Client for talking to auth0
#[derive(Clone)]
pub struct Auth0Client {
    token_url: Url,
    client: Client,
    client_id: String,
    client_secret: String,
}

impl Auth0Client {
    pub fn new(token_url: Url, client: Client, client_id: String, client_secret: String) -> Self {
        Self {
            token_url,
            client,
            client_id,
            client_secret,
        }
    }

    pub(super) fn from_config(config: &Config, client: &Client) -> Self {
        Auth0Client::new(
            config.token_url().clone(),
            client.clone(),
            config.client_id().to_string(),
            config.client_secret().to_string(),
        )
    }

    pub async fn fetch_token(&self, audience: &str, scope: Option<&str>) -> Result<Token, Auth0Error> {
        let request: FetchTokenRequest = FetchTokenRequest {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            audience: audience.to_string(),
            grant_type: "client_credentials".to_string(),
            scope: scope.map(str::to_string),
        };

        let response: Response = self
            .client
            .post(self.token_url.clone())
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                Auth0Error::JwtFetchError(
                    e.status().map(|v| v.as_u16()).unwrap_or_default(),
                    self.token_url.to_string(),
                    e,
                )
            })?;

        let status_code: u16 = response.status().as_u16();
        if status_code != 200 {
            return Err(Auth0Error::JwtFetchAuthError(status_code));
        }

        let response: FetchTokenResponse = response
            .json()
            .await
            .map_err(|e| Auth0Error::JwtFetchDeserializationError(self.token_url.as_str().to_string(), e))?;

        let FetchTokenResponse {
            access_token,
            expires_in,
            ..
        } = response;

        // this is not the exact issue_date, nor the exact expire_date. But is a good approximation
        // as long as we need it just to remove the key from the cache, and calculate the approximation
        // of the token lifetime. If we need more correctness we can decrypt the token and get
        // the exact issued_at (iat) and expiration (exp)
        // reference: https://www.iana.org/assignments/jwt/jwt.xhtml
        let issue_date: DateTime<Utc> = Utc::now();
        let expire_date: DateTime<Utc> = Utc::now() + Duration::from_secs(expires_in as u64);

        Ok(Token::new(access_token, issue_date, expire_date))
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}
