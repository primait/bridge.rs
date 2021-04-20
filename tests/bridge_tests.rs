use reqwest::Url;

use prima_bridge::Bridge;

mod common;

#[cfg(not(feature = "blocking"))]
mod async_bridge;
#[cfg(feature = "blocking")]
mod blocking;

#[cfg(feature = "auth0")]
const BRIDGE_ERROR: &str = "Cannot create new bridge";

pub struct Generator;

impl Generator {
    #[cfg(all(feature = "blocking", feature = "auth0"))]
    pub fn bridge(url: Url) -> Bridge {
        Bridge::new(url, Self::auth0_config()).expect(BRIDGE_ERROR)
    }

    #[cfg(all(feature = "blocking", not(feature = "auth0")))]
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

    #[cfg(all(feature = "blocking", feature = "auth0"))]
    pub fn bridge_with_user_agent(url: Url, user_agent: &str) -> Bridge {
        Bridge::with_user_agent(url, user_agent, Self::auth0_config()).expect(BRIDGE_ERROR)
    }

    #[cfg(all(feature = "blocking", not(feature = "auth0")))]
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

    #[cfg(feature = "auth0")]
    pub fn auth0_config() -> prima_bridge::auth0_config::Auth0Config {
        use std::str::FromStr;
        prima_bridge::auth0_config::Auth0Config::new(
            Url::from_str("http://should.be/mock/url").unwrap(),
            "audience".to_string(),
            "none".to_string(),
            "caller".to_string(),
            "32char_long_token_encryption_key".to_string(),
            std::time::Duration::from_secs(10),
            10,
            1000,
            "client_id".to_string(),
            "client_secret".to_string(),
            Url::from_str("http://should.be/mock/url_for_jwks").unwrap(),
        )
    }
}
