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
                let (send, recv) = oneshot::channel();
                let _ = interval_sender
                    .send(TokenDispenserMessage::TokenNeedsRefresh { respond_to: send })
                    .await;
                let needs_refresh = recv.await.expect("task has been killed");
                if needs_refresh {
                    let _ = interval_sender
                        .send(TokenDispenserMessage::RefreshToken)
                        .await;
                }
            }
        });
    }

    pub async fn refresh_token(&self) {
        let msg = TokenDispenserMessage::RefreshToken;
        let _ = self.sender.send(msg).await;
    }

    pub async fn get_token(&self) -> Option<String> {
        let (send, recv) = oneshot::channel();
        let msg = TokenDispenserMessage::GetToken { respond_to: send };
        let _ = self.sender.send(msg).await;
        recv.await.expect("get_token task has been killed")
    }
}
