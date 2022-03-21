use reqwest::Url;

/// auth0 related functions
#[cfg(feature = "auth0")]
use crate::auth0;
use crate::Bridge;
use recloser::AsyncRecloser;
use recloser::Recloser;

/// A builder for creating Bridge instances
pub struct BridgeBuilder {
    client_builder: reqwest::ClientBuilder,
    #[cfg(feature = "auth0")]
    auth0: Option<auth0::Auth0>,
    circuit_breaker: Option<Recloser>,
}

impl BridgeBuilder {
    pub(crate) fn create() -> Self {
        Self {
            client_builder: reqwest::ClientBuilder::new(),
            #[cfg(feature = "auth0")]
            auth0: None,
            circuit_breaker: None,
        }
    }

    pub fn with_user_agent(self, user_agent: impl Into<String>) -> Self {
        let builder = Self {
            client_builder: self.client_builder.user_agent(user_agent.into().as_str()),
            ..self
        };

        builder
    }

    // add auth0 capabilities to this bridge
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

    pub fn with_circuit_breaker(self, recloser: Recloser) -> Self {
        Self {
            circuit_breaker: Some(recloser),
            ..self
        }
    }

    pub fn build(self, endpoint: Url) -> Bridge {
        Bridge {
            client: self
                .client_builder
                .build()
                .expect("Unable to create Bridge"),
            endpoint,
            #[cfg(feature = "auth0")]
            auth0_opt: self.auth0,
            circuit_breaker: self.circuit_breaker.map(AsyncRecloser::from),
        }
    }
}
