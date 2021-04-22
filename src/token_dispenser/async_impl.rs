use std::time::Duration;

use chrono::Utc;
use reqwest::Url;
use tokio::sync::{mpsc, oneshot};

use crate::auth0_config::Auth0Config;
use crate::cache::{Cache, CacheEntry, Cacher};
use crate::errors::PrimaBridgeResult;
use crate::token_dispenser::jwks::{TokenChecker, TokenCheckerMsg};

#[derive(Debug)]
pub struct TokenDispenserHandle {
    sender: mpsc::Sender<TokenDispenserMessage>,
    sender_jwks_checker: mpsc::Sender<TokenCheckerMsg>,
}

/// the handle is responsible of exposing a public api to the underlying actor
impl TokenDispenserHandle {
    pub fn run(
        http_client: &reqwest::Client,
        cache: &Cache,
        config: Auth0Config,
    ) -> PrimaBridgeResult<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let mut actor = TokenDispenser::new(
            receiver,
            http_client,
            cache,
            config.base_url(),
            "caller",
            config.audience(),
            config.min_token_remaining_life_percentage(),
            config.max_token_remaining_life_percentage(),
        );
        tokio::spawn(async move { actor.run().await });

        let (jwks_token_checker_sender, jwks_token_checker_receiver) = mpsc::channel(8);
        let mut jwks_checker = TokenChecker::new(
            jwks_token_checker_receiver,
            http_client.clone(),
            config.jwks_url(),
        );
        tokio::spawn(async move { jwks_checker.run().await });

        Ok(Self {
            sender,
            sender_jwks_checker: jwks_token_checker_sender,
        })
    }

    pub async fn periodic_check(&self, duration: std::time::Duration) {
        let interval_sender = self.sender.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(duration);
            loop {
                ticker.tick().await;
                if Self::needs_refresh(&interval_sender).await {
                    Self::do_cast(&interval_sender, TokenDispenserMessage::RefreshToken).await;
                }
            }
        });
    }

    pub async fn fetch_jwks(&self) {
        let _ = self.sender_jwks_checker.send(TokenCheckerMsg::FetchJwks);
    }

    pub async fn refresh_token(&self) {
        self.cast(TokenDispenserMessage::RefreshToken).await;
    }

    pub async fn get_token(&self) -> Option<String> {
        self.retrieve_value(TokenDispenserMessage::get_token).await
    }

    async fn needs_refresh(sender: &mpsc::Sender<TokenDispenserMessage>) -> bool {
        Self::do_retrieve_value(sender, TokenDispenserMessage::token_needs_refresh).await
    }

    async fn retrieve_value<T>(
        &self,
        msg: impl Fn(oneshot::Sender<T>) -> TokenDispenserMessage,
    ) -> T {
        Self::do_retrieve_value(&self.sender, msg).await
    }

    async fn do_retrieve_value<T>(
        sender: &mpsc::Sender<TokenDispenserMessage>,
        msg: impl Fn(oneshot::Sender<T>) -> TokenDispenserMessage,
    ) -> T {
        let (send, recv) = oneshot::channel();
        let _ = sender.send(msg(send)).await;
        recv.await.expect("get_token task has been killed")
    }

    async fn cast(&self, msg: TokenDispenserMessage) {
        Self::do_cast(&self.sender, msg).await;
    }

    async fn do_cast(sender: &mpsc::Sender<TokenDispenserMessage>, msg: TokenDispenserMessage) {
        let _ = sender.send(msg).await;
    }
}

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
    cache: Cache,
    endpoint: Url,
    key: String,
    token: Option<CacheEntry>,
    min_token_remaining_life_percentage: i32,
    max_token_remaining_life_percentage: i32,
}

impl TokenDispenser {
    pub fn new(
        receiver: mpsc::Receiver<TokenDispenserMessage>,
        http_client: &reqwest::Client,
        cache: &Cache,
        endpoint: &Url,
        // Todo: better naming
        caller: &str,
        audience: &str,
        min_token_remaining_life_percentage: i32,
        max_token_remaining_life_percentage: i32,
    ) -> Self {
        Self {
            receiver,
            http_client: http_client.clone(),
            cache: cache.clone(),
            key: format!("auth0rs_tokens:{}:{}", caller, audience),
            endpoint: endpoint.clone(),
            token: None,
            min_token_remaining_life_percentage,
            max_token_remaining_life_percentage,
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
        // Get token from auth0
        let response: super::TokenResponse = self
            .http_client
            .get(self.endpoint.clone())
            .send()
            .await?
            .json()
            .await?;

        // todo: decompose token with jwk and create entry getting issue and expire dates.
        let token: CacheEntry = CacheEntry::new(response.token, Utc::now(), Utc::now());
        self.token = Some(token.clone());
        // Putting token in 2nd level cache
        self.cache.set(&self.key, token)
    }

    // Check if the token remaining lifetime it's less than a randomized percentage that is between
    // `max_token_remaining_life_percentage` and `min_token_remaining_life_percentage`
    fn token_needs_refresh(&self) -> bool {
        match &self.token {
            Some(entry) => {
                let random_percentage = random(
                    self.max_token_remaining_life_percentage as f64,
                    self.min_token_remaining_life_percentage as f64,
                );
                random_percentage >= entry.remaining_life_percentage()
            }
            None => true,
        }
    }
}

fn random(x: f64, y: f64) -> f64 {
    use rand::Rng;
    match x - y {
        z if z == 0.0 => x,
        z if z > 0.0 => rand::thread_rng().gen_range(y..x),
        _ => rand::thread_rng().gen_range(x..y),
    }
}

#[cfg(test)]
mod tests {
    use crate::token_dispenser::async_impl::random;

    #[test]
    fn random_test() {
        // Should never panic
        let _ = random(0.0, 1.0);
        let _ = random(1.0, 0.0);
        assert_eq!(random(0.0, 0.0), 0.0);
    }
}
