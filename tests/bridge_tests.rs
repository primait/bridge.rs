use reqwest::Url;

use prima_bridge::Bridge;

mod common;

#[cfg(not(feature = "blocking"))]
mod async_bridge;
#[cfg(feature = "blocking")]
mod blocking;

#[cfg(all(not(feature = "blocking"), feature = "auth0"))]
const BRIDGE_ERROR: &str = "Cannot create new bridge";

pub struct Generator;

impl Generator {
    #[cfg(feature = "blocking")]
    pub fn bridge(url: Url) -> Bridge {
        Bridge::new(url)
    }

    #[cfg(all(not(feature = "blocking"), feature = "auth0"))]
    pub async fn bridge(url: Url) -> Bridge {
        Bridge::new(url, Self::auth0_config())
            .await
            .expect(BRIDGE_ERROR)
    }

    #[cfg(all(not(feature = "blocking"), not(feature = "auth0")))]
    pub async fn bridge(url: Url) -> Bridge {
        Bridge::new(url)
    }

    #[cfg(all(feature = "blocking"))]
    pub fn bridge_with_user_agent(url: Url, user_agent: &str) -> Bridge {
        Bridge::with_user_agent(url, user_agent)
    }

    #[cfg(all(not(feature = "blocking"), feature = "auth0"))]
    pub async fn bridge_with_user_agent(url: Url, user_agent: &str) -> Bridge {
        Bridge::with_user_agent(url, user_agent, Self::auth0_config())
            .await
            .expect(BRIDGE_ERROR)
    }

    #[cfg(all(not(feature = "blocking"), not(feature = "auth0")))]
    pub async fn bridge_with_user_agent(url: Url, user_agent: &str) -> Bridge {
        Bridge::with_user_agent(url, user_agent)
    }

    #[cfg(all(not(feature = "blocking"), feature = "auth0"))]
    pub async fn bridge_with_auth0_config(
        url: Url,
        auth0_config: prima_bridge::auth0_config::Auth0Config,
    ) -> Bridge {
        Bridge::new(url, auth0_config).await.expect(BRIDGE_ERROR)
    }

    #[cfg(all(not(feature = "blocking"), feature = "auth0"))]
    pub fn auth0_config() -> prima_bridge::auth0_config::Auth0Config {
        use std::str::FromStr;
        prima_bridge::auth0_config::Auth0Config::new(
            Url::from_str("http://should.be/token/mock/url").unwrap(),
            Url::from_str("http://should.be/jkws/mock/url").unwrap(),
            "caller".to_string(),
            "audience".to_string(),
            "none".to_string(),
            "32char_long_token_encryption_key".to_string(),
            std::time::Duration::from_secs(10),
            20,
            30,
            "client_id".to_string(),
            "client_secret".to_string(),
        )
    }
}
