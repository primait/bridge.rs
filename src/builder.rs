use reqwest::Url;

/// auth0 related functions
#[cfg(feature = "auth0")]
use crate::auth0;
use crate::Bridge;

/// A builder for creating Bridge instances
pub struct BridgeBuilder {
    #[cfg(feature = "blocking")]
    client_builder: reqwest::blocking::ClientBuilder,
    #[cfg(not(feature = "blocking"))]
    client_builder: reqwest::ClientBuilder,
    #[cfg(feature = "auth0")]
    auth0: Option<auth0::Auth0>,
}

impl BridgeBuilder {
    pub(crate) fn create() -> Self {
        Self {
            #[cfg(feature = "blocking")]
            client_builder: reqwest::blocking::ClientBuilder::new(),
            #[cfg(not(feature = "blocking"))]
            client_builder: reqwest::ClientBuilder::new(),
            #[cfg(feature = "auth0")]
            auth0: None,
        }
    }

    pub fn with_user_agent(self, user_agent: impl Into<String>) -> Self {
        Self {
            client_builder: self.client_builder.user_agent(user_agent.into().as_str()),
        }
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

    pub fn build(self, endpoint: Url) -> Bridge {
        Bridge {
            client: self
                .client_builder
                .build()
                .expect("Unable to create Bridge"),
            endpoint,
            #[cfg(feature = "auth0")]
            auth0_opt: self.auth0,
        }
    }
}
