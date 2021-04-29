use crate::prelude::{
    PrimaBridgeError::{Auth0JwksFetchError, Auth0JwksFetchInvalidJsonError},
    PrimaBridgeResult,
};

use alcoholic_jwt::{token_kid, JWKS};
use reqwest::Url;
use tokio::sync::{mpsc, oneshot};

pub enum TokenCheckerMsg {
    FetchJwks,
    Validate {
        token: JwtToken,
        respond_to: oneshot::Sender<bool>,
    },
}

impl TokenCheckerMsg {
    pub fn validate(token: impl Into<JwtToken>) -> impl FnOnce(oneshot::Sender<bool>) -> Self {
        move |respond_to| Self::Validate {
            token: token.into(),
            respond_to,
        }
    }
}

pub struct TokenChecker {
    http_client: reqwest::Client,
    jwks_url: Url,
    jwks: Option<JWKS>,
    receiver: mpsc::Receiver<TokenCheckerMsg>,
}

pub struct JwtToken(String);

impl JwtToken {
    fn get_kid(&self) -> Option<String> {
        token_kid(&self.0).ok().flatten()
    }

    pub fn is_signed_by(&self, jwks: &JWKS) -> bool {
        self.get_kid().and_then(|kid| jwks.find(&kid)).is_some()
    }
}

impl From<String> for JwtToken {
    fn from(token_string: String) -> Self {
        Self(token_string)
    }
}

impl TokenChecker {
    /// creates the token checker actor.
    /// This may fail because we try to fetch the jwks from auth0
    pub fn new(
        receiver: mpsc::Receiver<TokenCheckerMsg>,
        http_client: reqwest::Client,
        jwks_url: &Url,
    ) -> Self {
        Self {
            http_client,
            jwks_url: jwks_url.clone(),
            receiver,
            jwks: None,
        }
    }

    async fn handle_message(&mut self, msg: TokenCheckerMsg) {
        match msg {
            TokenCheckerMsg::FetchJwks => {
                // if the request, for any reason, fails, we simply skip the update
                // We already handle the "missing jwks" case by always responding true
                if let Some(jwks) = self.fetch_jwks().await.ok() {
                    self.jwks = Some(jwks);
                }
            }
            TokenCheckerMsg::Validate { token, respond_to } => {
                // if we don't have jwks, or the token doesn't have a key, we simply return true
                let _ = respond_to.send(
                    self.jwks
                        .as_ref()
                        .map(|jwks| token.is_signed_by(jwks))
                        .unwrap_or(true),
                );
            }
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }

    async fn fetch_jwks(&self) -> PrimaBridgeResult<JWKS> {
        self.http_client
            .get(self.jwks_url.clone())
            .send()
            .await
            .map_err(|e| Auth0JwksFetchError(self.jwks_url.clone(), e))?
            .json::<JWKS>()
            .await
            .map_err(|e| Auth0JwksFetchInvalidJsonError(self.jwks_url.clone(), e))
    }
}
