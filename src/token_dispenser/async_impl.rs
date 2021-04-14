use reqwest::Url;
use tokio::sync::{mpsc, oneshot};

use crate::auth0_configuration::Auth0Configuration;
use crate::cache::{Cache, Cacher};
use crate::errors::PrimaBridgeResult;

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
    endpoint: Url,
    audience: String,
    token: Option<String>,
    receiver: mpsc::Receiver<TokenDispenserMessage>,
    cache: Cache,
}

impl TokenDispenser {
    pub fn new(
        endpoint: Url,
        audience: String,
        receiver: mpsc::Receiver<TokenDispenserMessage>,
        cache: Cache,
    ) -> Self {
        Self {
            endpoint,
            audience,
            token: None,
            receiver,
            cache,
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }

    async fn handle_refresh(&mut self) -> reqwest::Result<()> {
        // todo: usare il client del bridge
        let response: Option<super::TokenResponse> =
            reqwest::get(self.endpoint.clone()).await?.json().await?;

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

#[derive(Debug)]
pub struct TokenDispenserHandle {
    sender: mpsc::Sender<TokenDispenserMessage>,
}

/// the handle is responsible of exposing a public api to the underlying actor
impl TokenDispenserHandle {
    pub fn run(config: Auth0Configuration) -> PrimaBridgeResult<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let mut actor = TokenDispenser::new(
            config.base_url().clone(),
            config.audience(),
            receiver,
            Cache::new(config.cache_config())?,
        );
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
