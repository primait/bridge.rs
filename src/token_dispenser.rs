use reqwest::Url;
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot};

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    token: String,
}

pub struct TokenDispenser {
    endpoint: Url,
    audience: String,
    token: Option<String>,
    receiver: mpsc::Receiver<TokenDispenserMessage>,
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

impl TokenDispenser {
    pub fn new(
        endpoint: Url,
        audience: String,
        receiver: mpsc::Receiver<TokenDispenserMessage>,
    ) -> Self {
        Self {
            endpoint,
            audience: audience.to_string(),
            token: None,
            receiver,
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }

    async fn handle_refresh(&mut self) -> reqwest::Result<()> {
        let response: Option<TokenResponse> =
            reqwest::get(self.endpoint.clone()).await?.json().await?;

        self.token = response.map(|token_response| token_response.token);
        Ok(())
    }

    async fn handle_message(&mut self, msg: TokenDispenserMessage) {
        match msg {
            TokenDispenserMessage::RefreshToken => {
                self.handle_refresh().await;
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

#[derive(Clone, Debug)]
pub struct TokenDispenserHandle {
    sender: mpsc::Sender<TokenDispenserMessage>,
}

impl TokenDispenserHandle {
    pub fn run(endpoint: Url, audience: impl ToString) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let mut actor = TokenDispenser::new(endpoint, audience.to_string(), receiver);
        tokio::spawn(async move { actor.run().await });

        Self { sender }
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
        self.call(TokenDispenserMessage::get_token).await
    }

    async fn needs_refresh(sender: &mpsc::Sender<TokenDispenserMessage>) -> bool {
        Self::do_call(sender, TokenDispenserMessage::token_needs_refresh).await
    }

    async fn call<T>(&self, msg: impl Fn(oneshot::Sender<T>) -> TokenDispenserMessage) -> T {
        Self::do_call(&self.sender, msg).await
    }

    async fn do_call<T>(
        sender: &mpsc::Sender<TokenDispenserMessage>,
        msg: impl Fn(oneshot::Sender<T>) -> TokenDispenserMessage,
    ) -> T {
        let (send, recv) = oneshot::channel();
        let _ = sender.send(msg(send)).await;
        recv.await.expect("get_token task has been killed")
    }

    async fn cast(&self, msg: TokenDispenserMessage) {
        Self::do_cast(&self.sender, msg);
    }

    async fn do_cast(sender: &mpsc::Sender<TokenDispenserMessage>, msg: TokenDispenserMessage) {
        let _ = sender.send(msg).await;
    }
}
