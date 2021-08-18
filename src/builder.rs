use crate::Bridge;
use reqwest::Url;

#[cfg(feature = "blocking")]
pub struct BridgeBuilder {
    client_builder: reqwest::blocking::ClientBuilder,
    endpoint: Url,
}

#[cfg(feature = "blocking")]
impl BridgeBuilder {
    pub fn create(endpoint: Url) -> Self {
        Self {
            endpoint,
            client_builder: reqwest::blocking::ClientBuilder::new(),
        }
    }

    pub fn with_user_agent(self, user_agent: impl Into<String>) -> Self {
        Self {
            client_builder: self.client_builder.user_agent(user_agent.into().as_str()),
            ..self
        }
    }

    pub fn build(self) -> Bridge {
        Bridge {
            client: self
                .client_builder
                .build()
                .expect("Unable to create Bridge"),
            endpoint: self.endpoint,
        }
    }
}

#[cfg(not(feature = "blocking"))]
pub struct BridgeBuilder {
    client_builder: reqwest::ClientBuilder,
    endpoint: Url,
}

#[cfg(not(feature = "blocking"))]
impl BridgeBuilder {
    pub fn create(endpoint: Url) -> Self {
        Self {
            endpoint,
            client_builder: reqwest::ClientBuilder::new(),
        }
    }

    pub fn with_user_agent(self, user_agent: impl Into<String>) -> Self {
        Self {
            client_builder: self.client_builder.user_agent(user_agent.into().as_str()),
            ..self
        }
    }

    pub fn build(self) -> Bridge {
        Bridge {
            client: self
                .client_builder
                .build()
                .expect("Unable to create Bridge"),
            endpoint: self.endpoint,
        }
    }
}
