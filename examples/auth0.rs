//! Auth0 example with my configurations
//!
//! Before run the command to run this example be sure redis is running. Otherwise run this command:
//!
//! ```shell
//! docker run -p 6379:6379 --name bridge_redis -d redis
//! ```
//!
//! Export needed variables:
//! ```shell
//! export TOKEN_URL='https://{tenant}.auth0.com/oauth/token'
//! export JWKS_URL='https://{tenant}/.well-known/jwks.json'
//! export CLIENT_ID={client_id}
//! export CLIENT_SECRET={client_secret}
//! export AUDIENCE={audience}
//! ```
//!
//! And then to run this example execute:
//!
//! ```shell
//! cargo make auth0-example
//! ```

#[cfg(feature = "auth0")]
use prima_bridge::auth0::{Auth0, Auth0Error};

#[cfg(not(feature = "auth0"))]
fn main() {}

#[cfg(feature = "auth0")]
#[tokio::main]
async fn main() {
    use std::time::Duration;
    let client: reqwest::Client = reqwest::Client::new();
    let result: Result<Auth0, Auth0Error> = Auth0::new(&client, auth0::config()).await;

    match result {
        Err(error) => panic!("{}", error.to_string()),
        Ok(_auth0) => loop {
            std::thread::sleep(Duration::from_secs(10));
        },
    }
}

#[cfg(feature = "auth0")]
mod auth0 {
    use std::time::Duration;

    use prima_bridge::auth0::{CacheType, Config, StalenessCheckPercentage};

    pub fn config() -> Config {
        use reqwest::Url;
        use std::str::FromStr;

        let token_url: String = std::env::var("TOKEN_URL").unwrap();
        let jwks_url: String = std::env::var("JWKS_URL").unwrap();
        let client_id: String = std::env::var("CLIENT_ID").unwrap();
        let client_secret: String = std::env::var("CLIENT_SECRET").unwrap();
        let audience: String = std::env::var("AUDIENCE").unwrap();

        Config {
            token_url: Url::from_str(token_url.as_str()).unwrap(),
            jwks_url: Url::from_str(jwks_url.as_str()).unwrap(),
            caller: "paperboy".to_string(),
            audience,
            cache_type: CacheType::Inmemory,
            token_encryption_key: "32char_long_token_encryption_key".to_string(),
            check_interval: Duration::from_secs(2),
            staleness_check_percentage: StalenessCheckPercentage::new(0.1, 0.5),
            client_id,
            client_secret,
        }
    }
}
