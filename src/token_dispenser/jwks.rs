use crate::prelude::{
    PrimaBridgeError::{Auth0JWKSFetchError, Auth0JWKSFetchInvalidJsonError},
    PrimaBridgeResult,
};

use alcoholic_jwt::JWKS;
use reqwest::Url;
use tokio::sync::{mpsc, oneshot};

pub enum TokenCheckerMsg {
    FetchJwks,
    Check {
        token: String,
        respond_to: oneshot::Sender<bool>,
    },
}

pub struct TokenChecker {
    http_client: reqwest::Client,
    jwks_url: Url,
    jwks: Option<JWKS>,
    receiver: mpsc::Receiver<TokenCheckerMsg>,
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
                self.jwks = self.fetch_jwks().await.ok();
            }
            TokenCheckerMsg::Check { respond_to, .. } => {
                let _ = respond_to.send(true);
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
            .map_err(|e| Auth0JWKSFetchError(self.jwks_url.clone(), e))?
            .json::<JWKS>()
            .await
            .map_err(|e| Auth0JWKSFetchInvalidJsonError(self.jwks_url.clone(), e))
    }
}
