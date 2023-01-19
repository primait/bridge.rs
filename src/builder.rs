use std::sync::Arc;

use reqwest::Url;
use reqwest_middleware::Middleware;

#[cfg(feature = "auth0")]
use crate::auth0;
use crate::{Bridge, BridgeImpl, RedirectPolicy};

pub type BridgeBuilder = BridgeBuilderInner<reqwest::ClientBuilder>;

/// A builder for creating [Bridge] instances.
pub struct BridgeBuilderInner<T> {
    inner: T,
    #[cfg(feature = "auth0")]
    auth0: Option<auth0::Auth0>,
}

impl<T> BridgeBuilderInner<T> {
    /// Adds Auth0 JWT authentication to the requests made by the [Bridge].
    #[cfg_attr(docsrs, doc(cfg(feature = "auth0")))]
    #[cfg(feature = "auth0")]
    pub async fn with_auth0(self, config: auth0::Config) -> Self {
        let client: reqwest::Client = reqwest::Client::new();
        Self {
            auth0: Some(
                auth0::Auth0::new(&client, config)
                    .await
                    .expect("Failed to create auth0 bridge"),
            ),
            ..self
        }
    }
}

impl BridgeBuilderInner<reqwest::ClientBuilder> {
    pub(crate) fn create() -> Self {
        Self {
            inner: reqwest::ClientBuilder::new(),
            #[cfg(feature = "auth0")]
            auth0: None,
        }
    }

    pub fn with_user_agent(self, user_agent: impl Into<String>) -> Self {
        Self {
            inner: self.inner.user_agent(user_agent.into().as_str()),
            ..self
        }
    }

    pub fn with_redirect_policy(self, policy: RedirectPolicy) -> Self {
        Self {
            inner: self.inner.redirect(policy.into()),
            ..self
        }
    }

    pub fn with_pool_max_idle_per_host(self, max: usize) -> Self {
        Self {
            inner: self.inner.pool_max_idle_per_host(max),
            ..self
        }
    }

    pub fn with_middleware(self, layer: impl Middleware) -> BridgeBuilderInner<reqwest_middleware::ClientBuilder> {
        let client = self.inner.build().expect("Unable to create Bridge");
        BridgeBuilderInner {
            inner: reqwest_middleware::ClientBuilder::new(client).with(layer),
            #[cfg(feature = "auth0")]
            auth0: self.auth0,
        }
    }

    /// Creates a [Bridge] from this builder.
    ///
    /// The given endpoint will be the base URL of all the requests made by the Bridge.
    pub fn build(self, endpoint: Url) -> Bridge {
        Bridge {
            inner_client: self.inner.build().expect("Unable to create Bridge"),
            endpoint,
            #[cfg(feature = "auth0")]
            auth0_opt: self.auth0,
        }
    }
}

impl BridgeBuilderInner<reqwest_middleware::ClientBuilder> {
    pub fn with(self, layer: impl Middleware) -> Self {
        Self {
            inner: self.inner.with(layer),
            ..self
        }
    }

    pub fn with_arc(self, layer: Arc<dyn Middleware>) -> Self {
        Self {
            inner: self.inner.with_arc(layer),
            ..self
        }
    }

    pub fn build(self, endpoint: Url) -> BridgeImpl<reqwest_middleware::ClientWithMiddleware> {
        BridgeImpl {
            inner_client: self.inner.build(),
            endpoint,
            #[cfg(feature = "auth0")]
            auth0_opt: self.auth0,
        }
    }
}
