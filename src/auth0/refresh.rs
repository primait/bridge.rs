use std::error::Error;
use std::sync::Arc;
use std::sync::Weak;
use std::time::Duration;

use super::util::UnpoisonableRwLock;
use super::StalenessCheckPercentage;
use super::{client::Auth0Client, Auth0Error, Cache, Token};

#[derive(Debug, Clone)]
pub struct RefreshingToken(Arc<UnpoisonableRwLock<Token>>);

impl RefreshingToken {
    pub async fn new(
        client: Auth0Client,

        check_interval: Duration,
        staleness: StalenessCheckPercentage,
        cache: Box<dyn Cache>,

        audience: String,
        scope: Option<String>,
    ) -> Result<Self, crate::auth0::Auth0Error> {
        let token = client.fetch_token(&audience, scope.as_deref()).await?;
        let token = Arc::new(UnpoisonableRwLock::new(token));

        let worker = RefreshWorker {
            check_interval,
            client,
            cache,
            audience,
            staleness,
            scope,
        };

        tokio::spawn(worker.refresh_loop(Arc::downgrade(&token)));

        Ok(Self(token))
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<Token> {
        self.0.read()
    }
}

struct RefreshWorker {
    check_interval: Duration,
    client: Auth0Client,
    audience: String,
    scope: Option<String>,
    staleness: StalenessCheckPercentage,

    cache: Box<dyn Cache>,
}

impl RefreshWorker {
    async fn refresh_loop(self, weak_token: Weak<UnpoisonableRwLock<Token>>) {
        let mut ticker = tokio::time::interval(self.check_interval);

        loop {
            ticker.tick().await;

            let Some(token_rc) = weak_token.upgrade() else {
                tracing::debug!("All references to auth0 token dropped. Stopping refresh thread");
                return;
            };

            let token = token_rc.read().clone();
            match self.check_refresh_token(token).await {
                Ok(token) => token_rc.write(token),
                Err(e) => {
                    tracing::warn!(
                        error = &e as &dyn Error,
                        "Failed to refresh auth0 token: {e}! Will retry in {}s",
                        self.check_interval.as_secs()
                    )
                }
            };
        }
    }

    async fn check_refresh_token(&self, cur_token: Token) -> Result<Token, Auth0Error> {
        let cached_token = match self.cache.get_token().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error reading cached JWT. Reason: {:?}", e);
                None
            }
        };

        match cached_token {
            Some(cached_token) if cached_token.is_stale(&self.staleness) => {
                tracing::info!("Refreshing JWT and JWKS");

                let token = self.fetch_and_update_token().await?;
                Ok(token)
            }
            Some(cached_token) if cached_token.expire_date() > cur_token.expire_date() => Ok(cached_token),
            None => {
                tracing::debug!("New token expiry_date is lower current token. Not refreshing and trying to replace");
                self.cache.put_token(&cur_token).await?;
                Ok(cur_token)
            }
            _ => Ok(cur_token),
        }
    }

    // Unconditionally fetch a new token and update the cache
    async fn fetch_and_update_token(&self) -> Result<Token, Auth0Error> {
        let token: Token = self.client.fetch_token(&self.audience, self.scope.as_deref()).await?;

        if let Err(e) = self.cache.put_token(&token).await {
            tracing::error!(error = ?e, "JWT cache set failed: {e}");
        };

        Ok(token)
    }
}
