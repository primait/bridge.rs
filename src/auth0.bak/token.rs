use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::auth0::config::StalenessCheckPercentage;
use crate::errors::PrimaBridgeResult;
use crate::Config;

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

    pub async fn fetch(client: &Client, config: &Config) -> PrimaBridgeResult<Self> {
        let request: Request = Request::from(config);
        let response: Response = client
            .post(config.base_url().clone())
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        let access_token: String = response.access_token.clone();

        // this is not the exact issue_date, nor the exact expire_date. But is a good approximation
        // as long as we need it just to removes the key from the cache, and calculate the approximation
        // of the token lifetime. If we need more correctness we can decrypt the token and get
        // the exact issued_at (iat) and expiration (exp)
        // reference: https://www.iana.org/assignments/jwt/jwt.xhtml
        let issue_date: DateTime<Utc> = Utc::now();
        let expire_date: DateTime<Utc> = Utc::now() + Duration::seconds(response.expires_in as i64);

        Ok(Self {
            token: access_token,
            issue_date,
            expire_date,
        })
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn issue_date(&self) -> &DateTime<Utc> {
        &self.issue_date
    }

    pub fn expire_date(&self) -> &DateTime<Utc> {
        &self.expire_date
    }

    // Return the percentage of the remaining life
    pub fn remaining_life_percentage(&self) -> f64 {
        if *&self.expire_date.timestamp_millis() - Utc::now().timestamp_millis() <= 0 {
            0.0
        } else {
            let expire_millis: i64 = *&self.expire_date.timestamp_millis();
            let issue_millis: i64 = *&self.issue_date.timestamp_millis();
            let remaining: f64 = (expire_millis - Utc::now().timestamp_millis()) as f64;
            let total: f64 = (expire_millis - issue_millis) as f64;
            remaining / total * 100.0
        }
    }

    pub fn lifetime_in_seconds(&self) -> usize {
        (self.expire_date - self.issue_date).num_seconds() as usize
    }

    // Check if the token remaining lifetime it's less than a randomized percentage that is between
    // `max_token_remaining_life_percentage` and `min_token_remaining_life_percentage`
    pub fn needs_refresh(&self, config: &Config) -> bool {
        config.staleness_check_percentage().random_value_between()
            >= self.remaining_life_percentage()
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            token: "".to_string(),
            issue_date: Utc::now(),
            expire_date: Utc::now(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Response {
    access_token: String,
    scope: String,
    expires_in: i32,
    token_type: String,
}

#[derive(Serialize, Debug)]
struct Request {
    client_id: String,
    client_secret: String,
    audience: String,
    grant_type: String,
}

impl From<&Config> for Request {
    fn from(config: &Config) -> Self {
        Self {
            client_id: config.client_id().to_string(),
            client_secret: config.client_secret().to_string(),
            audience: config.audience().to_string(),
            grant_type: "client_credentials".to_string(),
        }
    }
}
