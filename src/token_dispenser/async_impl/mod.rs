use std::time::Duration;

use chrono::Utc;
use reqwest::Url;
use tokio::sync::{mpsc, oneshot};

use crate::auth0_config::Auth0Config;
use crate::cache::{Cache, CacheEntry};
use crate::errors::PrimaBridgeResult;
use crate::token_dispenser::async_impl::dispenser::{TokenDispenser, TokenDispenserMessage};
use crate::token_dispenser::async_impl::jwks::{TokenChecker, TokenCheckerMsg};
use alcoholic_jwt::ValidJWT;

mod dispenser;
mod jwks;

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
        config: &Auth0Config,
    ) -> PrimaBridgeResult<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let mut actor = TokenDispenser::new(receiver, http_client, cache, config);
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

    pub async fn periodic_jwks_check(&self, duration: std::time::Duration) {
        let token_dispenser_handle_sender = self.sender.clone();
        let jwks_interval_sender = self.sender_jwks_checker.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(duration);
            loop {
                ticker.tick().await;
                let maybe_token = Self::do_retrieve_value(
                    &token_dispenser_handle_sender,
                    TokenDispenserMessage::get_token,
                )
                .await;
                if let Some(token) = maybe_token {
                    let token_valid = Self::do_retrieve_value(
                        &jwks_interval_sender,
                        TokenCheckerMsg::check(token.clone()),
                    )
                    .await;

                    let claim: Option<ValidJWT> = Self::do_retrieve_value(
                        &jwks_interval_sender,
                        TokenCheckerMsg::validate(token),
                    )
                    .await;

                    if !token_valid {
                        Self::do_cast(
                            &token_dispenser_handle_sender,
                            TokenDispenserMessage::RefreshToken,
                        )
                        .await;
                    }
                }
            }
        });
    }

    pub async fn fetch_jwks(&self) {
        let _ = self
            .sender_jwks_checker
            .send(TokenCheckerMsg::FetchJwks)
            .await;
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

    async fn do_retrieve_value<T, M>(
        sender: &mpsc::Sender<M>,
        msg: impl FnOnce(oneshot::Sender<T>) -> M,
    ) -> T {
        let (send, recv) = oneshot::channel();
        let _ = sender.send(msg(send)).await;
        recv.await.expect("get_token task has been killed")
    }

    async fn cast(&self, msg: TokenDispenserMessage) {
        Self::do_cast(&self.sender, msg).await;
    }

    async fn do_cast<T>(sender: &mpsc::Sender<T>, msg: T) {
        let _ = sender.send(msg).await;
    }
}
