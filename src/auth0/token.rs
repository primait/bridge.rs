use chrono::{DateTime, Utc};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::auth0::errors::Auth0Error;
use crate::auth0::Config;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Token {
    token: String,
    issue_date: DateTime<Utc>,
    expire_date: DateTime<Utc>,
}

impl Token {
    pub fn new(token: String, issue_date: DateTime<Utc>, expire_date: DateTime<Utc>) -> Self {
        Self {
            token,
            issue_date,
            expire_date,
        }
    }

    pub async fn fetch(client_ref: &Client, config_ref: &Config) -> Result<Self, Auth0Error> {
        let request: FetchTokenRequest = FetchTokenRequest::from(config_ref);
        let response: Response = client_ref
            .post(config_ref.token_url().clone())
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                Auth0Error::JwtFetchError(
                    e.status().map(|v| v.as_u16()).unwrap_or_default(),
                    config_ref.token_url().as_str().to_string(),
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
            .map_err(|e| Auth0Error::JwtFetchDeserializationError(config_ref.token_url().as_str().to_string(), e))?;

        let access_token: String = response.access_token.clone();

        // this is not the exact issue_date, nor the exact expire_date. But is a good approximation
        // as long as we need it just to removes the key from the cache, and calculate the approximation
        // of the token lifetime. If we need more correctness we can decrypt the token and get
        // the exact issued_at (iat) and expiration (exp)
        // reference: https://www.iana.org/assignments/jwt/jwt.xhtml
        let issue_date: DateTime<Utc> = Utc::now();
        let expire_date: DateTime<Utc> = Utc::now() + Duration::from_secs(response.expires_in as u64);

        Ok(Self {
            token: access_token,
            issue_date,
            expire_date,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.token
    }

    pub fn issue_date(&self) -> &DateTime<Utc> {
        &self.issue_date
    }

    pub fn expire_date(&self) -> &DateTime<Utc> {
        &self.expire_date
    }

    // Return the percentage of the remaining life (from 0 to 1)
    pub fn remaining_life_percentage(&self) -> f64 {
        if self.expire_date.timestamp_millis() - Utc::now().timestamp_millis() <= 0 {
            0.0
        } else {
            let expire_millis: i64 = self.expire_date.timestamp_millis();
            let issue_millis: i64 = self.issue_date.timestamp_millis();
            let remaining: f64 = (expire_millis - Utc::now().timestamp_millis()) as f64;
            let total: f64 = (expire_millis - issue_millis) as f64;
            remaining / total
        }
    }

    pub fn lifetime_in_seconds(&self) -> usize {
        (self.expire_date - self.issue_date).num_seconds() as usize
    }

    // Check if the token remaining lifetime it's less than a randomized percentage that is between
    // `max_token_remaining_life_percentage` and `min_token_remaining_life_percentage`
    pub fn needs_refresh(&self, config_ref: &Config) -> bool {
        self.remaining_life_percentage() < config_ref.staleness_check_percentage().random_value_between()
    }

    pub fn to_bearer(&self) -> String {
        format!("Bearer {}", self.token.as_str())
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct FetchTokenResponse {
    access_token: String,
    scope: String,
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

impl From<&Config> for FetchTokenRequest {
    fn from(config: &Config) -> Self {
        Self {
            client_id: config.client_id().to_string(),
            client_secret: config.client_secret().to_string(),
            audience: config.audience().to_string(),
            grant_type: "client_credentials".to_string(),
            scope: config.scope.clone(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Claims {
    #[serde(default)]
    pub permissions: Vec<String>,
}
