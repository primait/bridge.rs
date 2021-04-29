use chrono::Utc;
use tokio::sync::{mpsc, oneshot};

use crate::auth0_config::Auth0Config;
use crate::cache::{Cache, CacheEntry, CacheImpl};
use crate::errors::PrimaBridgeResult;
use crate::token_dispenser::{random, TokenRequest, TokenResponse};

#[derive(Debug)]
pub enum TokenDispenserMessage {
    RefreshToken,
    TokenNeedsRefresh {
        respond_to: oneshot::Sender<bool>,
    },
    GetToken {
        respond_to: oneshot::Sender<Option<String>>,
    },
}

impl TokenDispenserMessage {
    pub fn get_token(respond_to: oneshot::Sender<Option<String>>) -> Self {
        Self::GetToken { respond_to }
    }

    pub fn token_needs_refresh(respond_to: oneshot::Sender<bool>) -> Self {
        Self::TokenNeedsRefresh { respond_to }
    }
}

pub struct TokenDispenser {
    receiver: mpsc::Receiver<TokenDispenserMessage>,
    http_client: reqwest::Client,
    cache: CacheImpl,
    config: Auth0Config,
    key: String,
    token: Option<CacheEntry>,
}

impl TokenDispenser {
    pub fn new(
        receiver: mpsc::Receiver<TokenDispenserMessage>,
        http_client: &reqwest::Client,
        cache: &CacheImpl,
        config: &Auth0Config,
    ) -> Self {
        Self {
            receiver,
            http_client: http_client.clone(),
            cache: cache.clone(),
            config: config.clone(),
            key: Self::get_key(config),
            token: None,
        }
    }

    pub async fn run(&mut self) {
        // If a token already exists in 2nd level cache then use that.
        if let Ok(Some(token)) = self.cache.get(&self.key.to_string()) {
            self.token = Some(token);
        } else {
            let _ = self.refresh_token().await;
        }

        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }

    pub fn get_key(config: &Auth0Config) -> String {
        format!("auth0rs_tokens:{}:{}", config.caller(), config.audience())
    }

    async fn handle_message(&mut self, msg: TokenDispenserMessage) {
        match msg {
            TokenDispenserMessage::RefreshToken => {
                let _ = self.handle_refresh().await;
            }
            TokenDispenserMessage::GetToken { respond_to } => {
                let _ = respond_to.send(
                    self.token
                        .as_ref()
                        .map(|entry| entry.token().to_string())
                        .to_owned(),
                );
            }
            TokenDispenserMessage::TokenNeedsRefresh { respond_to } => {
                let _ = respond_to.send(self.token_needs_refresh());
            }
        }
    }

    async fn handle_refresh(&mut self) -> PrimaBridgeResult<()> {
        // First of all check in 2nd level cache if exist a newer token
        let token: Option<CacheEntry> = self.cache.get(&self.key.to_string())?;
        if self.token_needs_refresh() {
            self.refresh_token().await
        } else {
            self.token = token;
            Ok(())
        }
    }

    async fn refresh_token(&mut self) -> PrimaBridgeResult<()> {
        let request: TokenRequest = TokenRequest::new(
            self.config.client_id().to_string(),
            self.config.client_secret().to_string(),
            self.config.audience().to_string(),
        );

        let response: TokenResponse = self
            .http_client
            .post(self.config.base_url().clone())
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        let access_token: String = response.access_token.clone();

        // todo: decode token with jwk and create entry getting issue and expire dates.
        let issue_date = Utc::now() - chrono::Duration::hours(3);
        let token: CacheEntry = CacheEntry::new(access_token, issue_date, Utc::now());
        self.token = Some(token.clone());
        // Put token in 2nd level cache
        self.cache.set(&self.key, token)
    }

    fn token_needs_refresh(&self) -> bool {
        token_needs_refresh(
            &self.token,
            self.config.max_token_remaining_life_percentage(),
            self.config.min_token_remaining_life_percentage(),
        )
    }
}

// Check if the token remaining lifetime it's less than a randomized percentage that is between
// `max_token_remaining_life_percentage` and `min_token_remaining_life_percentage`
fn token_needs_refresh(
    token: &Option<CacheEntry>,
    max_token_remaining_life_percentage: i32,
    min_token_remaining_life_percentage: i32,
) -> bool {
    match &token {
        Some(entry) => {
            let random_percentage = random(
                max_token_remaining_life_percentage as f64,
                min_token_remaining_life_percentage as f64,
            );
            random_percentage >= entry.remaining_life_percentage()
        }
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use mockito::{mock, Mock};
    use tokio::sync::{mpsc, oneshot};

    use crate::auth0_config::Auth0Config;
    use crate::cache::{Cache, CacheEntry, CacheImpl};
    use crate::token_dispenser::async_impl::dispenser::{
        token_needs_refresh, TokenDispenser, TokenDispenserMessage,
    };
    use crate::token_dispenser::{random, TokenResponse};

    #[test]
    fn random_test() {
        // Should never panic
        let _ = random(0.0, 1.0);
        let _ = random(1.0, 0.0);
        assert_eq!(random(0.0, 0.0), 0.0);
    }

    #[test]
    fn token_needs_refresh_test() {
        let now: i64 = Utc::now().timestamp_millis();
        let max_token_remaining_life_percentage: i32 = 50;
        let min_token_remaining_life_percentage: i32 = 10;

        // Token is going to expire
        let issue_date: DateTime<Utc> = Utc.timestamp_millis(now - 1_000_000);
        let expire_date: DateTime<Utc> = Utc.timestamp_millis(now);
        let entry = CacheEntry::new("token".to_string(), issue_date, expire_date);
        assert!(token_needs_refresh(
            &Some(entry),
            max_token_remaining_life_percentage,
            min_token_remaining_life_percentage
        ));

        // Token has been just emitted
        let issue_date: DateTime<Utc> = Utc.timestamp_millis(now);
        let expire_date: DateTime<Utc> = Utc.timestamp_millis(now + 10_000);
        let entry = CacheEntry::new("token".to_string(), issue_date, expire_date);
        assert!(!token_needs_refresh(
            &Some(entry),
            max_token_remaining_life_percentage,
            min_token_remaining_life_percentage
        ));

        // Token has expired long time ago. Not an issue
        let issue_date: DateTime<Utc> = Utc.timestamp_millis(now - 1_001_000);
        let expire_date: DateTime<Utc> = Utc.timestamp_millis(now - 1_000_000);
        let entry = CacheEntry::new("token".to_string(), issue_date, expire_date);
        assert!(token_needs_refresh(
            &Some(entry),
            max_token_remaining_life_percentage,
            min_token_remaining_life_percentage
        ))
    }

    #[tokio::test]
    async fn dispenser_get_token_from_auth0_test() {
        let _m = create_auth0_mock();

        let (sender, receiver): (
            mpsc::Sender<TokenDispenserMessage>,
            mpsc::Receiver<TokenDispenserMessage>,
        ) = mpsc::channel(8);

        let http_client = reqwest::Client::new();
        let config = &Auth0Config::test_config();
        let cache = CacheImpl::new(config).unwrap();

        let mut token = TokenDispenser::new(receiver, &http_client, &cache, config);
        tokio::spawn(async move { token.run().await });

        let (oneshot_sender, oneshot_receiver): (
            oneshot::Sender<Option<String>>,
            oneshot::Receiver<Option<String>>,
        ) = oneshot::channel();

        let message: TokenDispenserMessage = TokenDispenserMessage::GetToken {
            respond_to: oneshot_sender,
        };

        let _ = sender.send(message).await;
        let response = oneshot_receiver
            .await
            .expect("get_token task has been killed");

        assert!(response.is_some());
        assert_eq!(response.unwrap(), "abcdef");
    }

    #[tokio::test]
    async fn dispenser_find_token_in_cache_test() {
        let _mock = create_auth0_mock();

        let (sender, receiver): (
            mpsc::Sender<TokenDispenserMessage>,
            mpsc::Receiver<TokenDispenserMessage>,
        ) = mpsc::channel(8);

        let http_client = reqwest::Client::new();
        let config = &Auth0Config::test_config();
        let mut cache = CacheImpl::new(config).unwrap();
        cache
            .set(
                &TokenDispenser::get_key(config),
                CacheEntry::new("hello".to_string(), Utc::now(), Utc::now()),
            )
            .unwrap();

        let mut token = TokenDispenser::new(receiver, &http_client, &cache, config);
        let _ = tokio::spawn(async move { token.run().await });

        let (oneshot_sender, oneshot_receiver): (
            oneshot::Sender<Option<String>>,
            oneshot::Receiver<Option<String>>,
        ) = oneshot::channel();

        let message: TokenDispenserMessage = TokenDispenserMessage::GetToken {
            respond_to: oneshot_sender,
        };

        let _ = sender.send(message).await;
        let response = oneshot_receiver
            .await
            .expect("get_token task has been killed");

        assert!(response.is_some());
        assert_eq!(response.unwrap(), "hello");
    }

    pub fn create_auth0_mock() -> Mock {
        let token_response = TokenResponse {
            access_token: "abcdef".to_string(),
            scope: "test".to_string(),
            expires_in: 5,
            token_type: "test".to_string(),
        };
        mock("POST", "/token")
            .with_status(200)
            .with_body(serde_json::to_string(&token_response).unwrap())
            .create()
    }
}
