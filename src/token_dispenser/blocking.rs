use std::sync::mpsc;

use reqwest::Url;

use crate::auth0_config::Auth0Config;
use crate::cache::CacheImpl;
use crate::errors::PrimaBridgeResult;

#[derive(Clone, Debug)]
pub struct TokenDispenserHandle {
    sender: mpsc::Sender<TokenDispenserMessage>,
}

impl TokenDispenserHandle {
    pub fn run(
        http_client: &reqwest::blocking::Client,
        cache: &CacheImpl,
        config: &Auth0Config,
    ) -> PrimaBridgeResult<Self> {
        let (sender, receiver) = mpsc::channel();
        let mut actor = TokenDispenser::new(
            http_client,
            cache,
            config.base_url(),
            config.caller(),
            config.audience(),
            receiver,
        );
        std::thread::spawn(move || actor.run());

        Ok(Self { sender })
    }

    pub fn periodic_check(&self, duration: std::time::Duration) {
        let interval_sender = self.sender.clone();
        std::thread::spawn(move || loop {
            if Self::needs_refresh(&interval_sender) {
                Self::do_cast(&interval_sender, TokenDispenserMessage::RefreshToken);
            }
            std::thread::sleep(duration);
        });
    }

    pub fn get_token(&self) -> Option<String> {
        self.retrieve_value(TokenDispenserMessage::get_token)
            .flatten()
    }

    pub fn refresh_token(&self) {
        self.cast(TokenDispenserMessage::RefreshToken);
    }

    pub fn needs_refresh(sender: &mpsc::Sender<TokenDispenserMessage>) -> bool {
        Self::do_retrieve_value(sender, TokenDispenserMessage::token_needs_refresh)
            .unwrap_or_default()
    }

    fn retrieve_value<T>(
        &self,
        msg: impl Fn(mpsc::Sender<T>) -> TokenDispenserMessage,
    ) -> Option<T> {
        Self::do_retrieve_value(&self.sender, msg)
    }

    fn do_retrieve_value<T>(
        sender: &mpsc::Sender<TokenDispenserMessage>,
        msg: impl Fn(mpsc::Sender<T>) -> TokenDispenserMessage,
    ) -> Option<T> {
        let (tx, rx) = mpsc::channel();
        let _ = sender.send(msg(tx));
        rx.recv().ok()
    }

    fn cast(&self, msg: TokenDispenserMessage) {
        Self::do_cast(&self.sender, msg);
    }

    fn do_cast(sender: &mpsc::Sender<TokenDispenserMessage>, msg: TokenDispenserMessage) {
        let _ = sender.send(msg);
    }
}

#[derive(Debug)]
pub enum TokenDispenserMessage {
    RefreshToken,
    GetToken {
        respond_to: mpsc::Sender<Option<String>>,
    },
    TokenNeedsRefresh {
        respond_to: mpsc::Sender<bool>,
    },
}

impl TokenDispenserMessage {
    pub fn get_token(respond_to: mpsc::Sender<Option<String>>) -> Self {
        Self::GetToken { respond_to }
    }

    pub fn token_needs_refresh(respond_to: mpsc::Sender<bool>) -> Self {
        Self::TokenNeedsRefresh { respond_to }
    }
}

pub struct TokenDispenser {
    http_client: reqwest::blocking::Client,
    cache: CacheImpl,
    endpoint: Url,
    key: String,
    token: Option<String>,
    receiver: mpsc::Receiver<TokenDispenserMessage>,
}

impl TokenDispenser {
    pub fn new(
        http_client: &reqwest::blocking::Client,
        cache: &CacheImpl,
        endpoint: &Url,
        // Todo: better naming
        caller: &str,
        audience: &str,
        receiver: mpsc::Receiver<TokenDispenserMessage>,
    ) -> Self {
        Self {
            http_client: http_client.clone(),
            cache: cache.clone(),
            key: format!("auth0rs_tokens:{}:{}", caller, audience),
            endpoint: endpoint.clone(),
            token: None,
            receiver,
        }
    }

    pub fn run(&mut self) {
        while let Ok(msg) = self.receiver.recv() {
            self.handle_message(msg);
        }
    }

    fn handle_refresh(&mut self) -> reqwest::Result<()> {
        let response: Option<super::TokenResponse> =
            self.http_client.get(self.endpoint.clone()).send()?.json()?;

        self.token = response.map(|token_response| token_response.access_token);

        // todo: cache set value here?
        // &self.cache.set()

        Ok(())
    }

    fn handle_message(&mut self, msg: TokenDispenserMessage) {
        match msg {
            TokenDispenserMessage::RefreshToken => {
                let _ = self.handle_refresh();
            }
            TokenDispenserMessage::GetToken { respond_to } => {
                let _ = respond_to.send(self.token.clone());
            }
            TokenDispenserMessage::TokenNeedsRefresh { respond_to } => {
                let _ = respond_to.send(true);
            }
        }
    }
}
