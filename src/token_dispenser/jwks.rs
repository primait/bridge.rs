use crate::prelude::{
    PrimaBridgeError::{Auth0JWKSFetchError, Auth0JWKSFetchInvalidJsonError},
    PrimaBridgeResult,
};

use alcoholic_jwt::JWKS;
use reqwest::Url;
use tokio::sync::{mpsc, oneshot};

pub enum TokenCheckerMsg {
    Check {
        token: String,
        respond_to: oneshot::Sender<bool>,
    },
}

pub struct TokenChecker {
    http_client: reqwest::Client,
    jwks: Option<JWKS>,
    receiver: mpsc::Receiver<TokenCheckerMsg>,
}

impl TokenChecker {
    /// creates the token checker actor.
    /// This may fail because we try to fetch the jwks from auth0
    pub async fn new(
        receiver: mpsc::Receiver<TokenCheckerMsg>,
        http_client: reqwest::Client,
        jwks_url: &Url,
    ) -> PrimaBridgeResult<Self> {
        let jwks = Self::fetch_jwks(http_client.clone(), jwks_url).await?;
        Ok(Self {
            http_client,
            receiver,
            jwks: Some(jwks),
        })
    }

    async fn handle_message(&mut self, msg: TokenCheckerMsg) {
        match msg {
            TokenCheckerMsg::Check { token, respond_to } => {
                respond_to.send(true);
            }
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }

    async fn fetch_jwks(http_client: reqwest::Client, jwks_url: &Url) -> PrimaBridgeResult<JWKS> {
        http_client
            .get(jwks_url.clone())
            .send()
            .await
            .map_err(|e| Auth0JWKSFetchError(jwks_url.clone(), e))?
            .json::<JWKS>()
            .await
            .map_err(|e| Auth0JWKSFetchInvalidJsonError(jwks_url.clone(), e))
    }
}
