use crate::Bridge;
use reqwest::Url;

pub struct BridgeBuilder {
    #[cfg(feature = "blocking")]
    client_builder: reqwest::blocking::ClientBuilder,
    #[cfg(not(feature = "blocking"))]
    client_builder: reqwest::ClientBuilder,
}

impl BridgeBuilder {
    pub(crate) fn create() -> Self {
        Self {
            #[cfg(feature = "blocking")]
            client_builder: reqwest::blocking::ClientBuilder::new(),
            #[cfg(not(feature = "blocking"))]
            client_builder: reqwest::ClientBuilder::new(),
        }
    }

    pub fn with_user_agent(self, user_agent: impl Into<String>) -> Self {
        Self {
            client_builder: self.client_builder.user_agent(user_agent.into().as_str()),
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
        }
    }
}
