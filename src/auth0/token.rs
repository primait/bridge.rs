use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::auth0::client::Auth0Client;
use crate::auth0::errors::Auth0Error;
use crate::auth0::Config;

use super::StalenessCheckPercentage;

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

    #[deprecated(since = "0.21.0", note = "please use Auth0Client instead")]
    pub async fn fetch(client: &Client, config: &Config) -> Result<Self, Auth0Error> {
        Auth0Client::new(
            config.token_url().clone(),
            client.clone(),
            config.client_id().to_string(),
            config.client_secret().to_string(),
        )
        .fetch_token(config.audience(), config.scope.as_deref())
        .await
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

    pub fn lifetime_in_seconds(&self) -> u64 {
        (self.expire_date - self.issue_date).num_seconds() as u64
    }

    // Check if the token remaining lifetime it's less than a randomized percentage that is between
    // `max_token_remaining_life_percentage` and `min_token_remaining_life_percentage`
    #[deprecated(since = "0.21.0", note = "please use the is_stale function instead")]
    pub fn needs_refresh(&self, config: &Config) -> bool {
        self.is_stale(config.staleness_check_percentage())
    }

    pub fn is_stale(&self, staleness: &StalenessCheckPercentage) -> bool {
        self.remaining_life_percentage() < staleness.random_value_between()
    }

    pub fn to_bearer(&self) -> String {
        format!("Bearer {}", self.token.as_str())
    }
}
