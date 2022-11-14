use reqwest::Url;

#[cfg(feature = "auth0")]
use crate::auth0;
use crate::{Bridge, RedirectPolicy};

/// A builder for creating [Bridge] instances.
pub struct BridgeBuilder {
    client_builder: reqwest::ClientBuilder,
    #[cfg(feature = "auth0")]
    auth0: Option<auth0::Auth0>,
}

impl BridgeBuilder {
    pub(crate) fn create() -> Self {
        Self {
            client_builder: reqwest::ClientBuilder::new(),
            #[cfg(feature = "auth0")]
            auth0: None,
        }
    }

    pub fn with_user_agent(self, user_agent: impl Into<String>) -> Self {
        Self {
            client_builder: self.client_builder.user_agent(user_agent.into().as_str()),
            ..self
        }
    }

    pub fn with_redirect_policy(self, policy: RedirectPolicy) -> Self {
        Self {
            client_builder: self.client_builder.redirect(policy.into()),
            ..self
        }
    }

    pub fn with_pool_max_idle_per_host(self, max: usize) -> Self {
        Self {
            client_builder: self.client_builder.pool_max_idle_per_host(max),
            ..self
        }
    }

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

    /// Creates a [Bridge] from this builder.
    ///
    /// The given endpoint will be the base URL of all the requests made by the Bridge.
    pub fn build(self, endpoint: Url) -> Bridge {
        Bridge {
            client: self.client_builder.build().expect("Unable to create Bridge"),
            endpoint,
            #[cfg(feature = "auth0")]
            auth0_opt: self.auth0,
        }
    }
}
