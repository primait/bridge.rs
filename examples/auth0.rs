//! Auth0 example with my configurations
//!
//! Before run the command to run this example be sure redis is running. Otherwise run this command:
//!
//! ```shell
//! docker run -p 6379:6379 --name bridge_redis -d redis
//! ```
//!
//! to run this example execute:
//!
//! ```shell
//! cargo make auth0-example
//! ```

use std::time::Duration;

use prima_bridge::auth0::{Auth0, Auth0Error};

const ENCRYPTION_KEY: &str = "32char_long_token_encryption_key";

#[cfg(feature = "auth0")]
#[tokio::main]
async fn main() {
    let client: reqwest::Client = reqwest::Client::new();
    let result: Result<Auth0, Auth0Error> = Auth0::new(&client, auth0::config()).await;

    match result {
        Err(error) => panic!("{}", error.to_string()),
        Ok(_) => loop {
            std::thread::sleep(Duration::from_secs(60 * 60));
        },
    }
}

#[cfg(feature = "auth0")]
mod auth0 {
    use std::time::Duration;

    use prima_bridge::auth0::{Config, StalenessCheckPercentage};

    use crate::*;

    pub fn config() -> Config {
        use reqwest::Url;
        use std::str::FromStr;

        let token_url: String = std::env::var("TOKEN_URL").unwrap();
        let jwks_url: String = std::env::var("JWKS_URL").unwrap();
        let client_id: String = std::env::var("CLIENT_ID").unwrap();
        let client_secret: String = std::env::var("CLIENT_SECRET").unwrap();

        Config::new(
            Url::from_str(token_url.as_str()).unwrap(),
            Url::from_str(jwks_url.as_str()).unwrap(),
            "paperboy".to_string(),
            "Paperboy".to_string(),
            "redis://localhost/".to_string(),
            ENCRYPTION_KEY.to_string(),
            Duration::from_secs(2),
            StalenessCheckPercentage::new(0.1, 0.5),
            client_id,
            client_secret,
        )
    }
}
