use reqwest::Url;

#[cfg(feature = "auth0")]
use crate::auth0;
use crate::Bridge;

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
            ..self
        }
    }

    // #[cfg(feature = "auth0")]
    // pub async fn with_auth0(self, config: auth0::Config) -> Self {
    //     Self {
    //         auth0: Some(auth0::Auth0::new(config).await),
    //         ..self
    //     }
    // }

    pub fn build(self, endpoint: Url) -> Bridge {
        Bridge {
            client: self
                .client_builder
                .build()
                .expect("Unable to create Bridge"),
            endpoint,
        }
    }
}
