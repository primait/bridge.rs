use reqwest::Url;
use tokio::sync::{mpsc, oneshot};

use crate::auth0_config::Auth0Config;
use crate::cache::{Cache, Cacher};
use crate::errors::PrimaBridgeResult;

#[derive(Debug)]
pub struct TokenDispenserHandle {
    sender: mpsc::Sender<TokenDispenserMessage>,
}

/// the handle is responsible of exposing a public api to the underlying actor
impl TokenDispenserHandle {
    pub fn run(http_client: reqwest::Client, config: Auth0Config) -> PrimaBridgeResult<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let mut actor = TokenDispenser {
            http_client: http_client.clone(),
            endpoint: config.base_url().to_owned(),
            audience: config.audience().to_string(),
            token: None,
            receiver,
            cache: Cache::new(&config)?,
        };
        tokio::spawn(async move { actor.run().await });

        Ok(Self { sender })
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
    pub http_client: reqwest::Client,
    pub endpoint: Url,
    pub audience: String,
    pub token: Option<String>,
    pub receiver: mpsc::Receiver<TokenDispenserMessage>,
    pub cache: Cache,
}

impl TokenDispenser {
    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }

    async fn handle_refresh(&mut self) -> reqwest::Result<()> {
        let response: Option<super::TokenResponse> = self
            .http_client
            .get(self.endpoint.clone())
            .send()
            .await?
            .json()
            .await?;

        self.token = response.map(|token_response| token_response.token);

        // todo: cache set value here?
        // &self.cache.set()

        Ok(())
    }

    async fn handle_message(&mut self, msg: TokenDispenserMessage) {
        match msg {
            TokenDispenserMessage::RefreshToken => {
                let _ = self.handle_refresh().await;
            }
            TokenDispenserMessage::GetToken { respond_to } => {
                let _ = respond_to.send(self.token.to_owned());
            }
            TokenDispenserMessage::TokenNeedsRefresh { respond_to } => {
                let _ = respond_to.send(true);
            }
        }
    }
}
