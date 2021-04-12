use reqwest::Url;
use std::sync::mpsc;

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
    endpoint: Url,
    audience: String,
    token: Option<String>,
    receiver: mpsc::Receiver<TokenDispenserMessage>,
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

    pub fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().ok() {
            self.handle_message(msg);
        }
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

    fn handle_refresh(&mut self) -> reqwest::Result<()> {
        let response: Option<super::TokenResponse> =
            reqwest::blocking::get(self.endpoint.clone())?.json()?;

        self.token = response.map(|token_response| token_response.token);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TokenDispenserHandle {
    sender: mpsc::Sender<TokenDispenserMessage>,
}

impl TokenDispenserHandle {
    pub fn run(endpoint: Url, audience: impl ToString) -> Self {
        let (sender, receiver) = mpsc::channel();
        let mut actor = TokenDispenser::new(endpoint, audience.to_string(), receiver);
        std::thread::spawn(move || actor.run());

        Self { sender }
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
