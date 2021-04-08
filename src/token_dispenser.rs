use crate::token_dispenser::TokenDispenserMessage::RefreshToken;
use reqwest::{Error, Response, Url};
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

pub enum TokenDispenserMessage {
    RefreshToken {
        respond_to: oneshot::Sender<Option<String>>,
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

    async fn handle_message(&mut self, msg: TokenDispenserMessage) {
        match msg {
            TokenDispenserMessage::RefreshToken { respond_to } => {
                let response = reqwest::get(self.endpoint.clone()).await;
                let token_response: Option<TokenResponse> = match response {
                    Ok(resp) => resp.json().await.ok(),
                    Err(_) => None,
                };
                self.token = token_response.map(|token_response| token_response.token);
                let _ = respond_to.send(self.token.clone());
            }
            TokenDispenserMessage::GetToken { respond_to } => {
                let _ = respond_to.send(self.token.to_owned());
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

    pub async fn refresh_token(&mut self) {
        let (send, recv) = oneshot::channel();
        let msg = TokenDispenserMessage::RefreshToken { respond_to: send };
        let _ = self.sender.send(msg).await;
        let _ = recv.await.expect("refresh_token task has been killed");
    }

    pub async fn get_token(&self) -> Option<String> {
        let (send, recv) = oneshot::channel();
        let msg = TokenDispenserMessage::GetToken { respond_to: send };
        let _ = self.sender.send(msg).await;
        recv.await.expect("get_token task has been killed")
    }
}
