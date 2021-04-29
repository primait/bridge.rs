// To run this example:
//
// export TOKEN_URL=https://{tenant}.eu.auth0.com/oauth/token
// export JWK_URL=https://{tenant}.eu.auth0.com/.well-known/jwks.json
// export CLIENT_ID=
// export CLIENT_SECRET=
//
// $ cargo run --example example --features="auth0 inmemory"
#[tokio::main]
#[cfg(all(not(feature = "blocking"), feature = "auth0", feature = "inmemory"))]
async fn main() {
    use prima_bridge::Bridge;
    use reqwest::Url;
    use std::str::FromStr;

    let _m = app::example_app_mock();
    let url: Url =
        Url::from_str(&format!("{}/{}", mockito::server_url().as_str(), "create")).unwrap();
    let bridge: Bridge = Bridge::new(url, app::auth0_config()).await.unwrap();

    loop {
        dbg!(bridge.get_headers().await.len());
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

#[cfg(not(all(not(feature = "blocking"), feature = "auth0", feature = "inmemory")))]
fn main() {}

#[cfg(all(feature = "auth0", feature = "inmemory"))]
mod app {
    use prima_bridge::auth0_config::Auth0Config;
    use reqwest::Url;
    use std::str::FromStr;
    use std::time::Duration;

    pub fn example_app_mock() -> mockito::Mock {
        mockito::mock("GET", "/create")
            .with_status(200)
            .with_body("{\"created\": \"true\"}")
            .create()
    }

    pub fn auth0_config() -> Auth0Config {
        let token_url: String = std::env::var("TOKEN_URL").unwrap();
        let jwk_url: String = std::env::var("JWK_URL").unwrap();
        let client_id: String = std::env::var("CLIENT_ID").unwrap();
        let client_secret: String = std::env::var("CLIENT_SECRET").unwrap();

        Auth0Config::new(
            Url::from_str(token_url.as_str()).unwrap(),
            Url::from_str(jwk_url.as_str()).unwrap(),
            "me".to_string(),
            "https://dev-8aes9g04.eu.auth0.com/api/v2/".to_string(),
            "inmemory".to_string(),
            "32char_long_token_encryption_key".to_string(),
            Duration::from_secs(2),
            10,
            50,
            client_id,
            client_secret,
        )
    }
}
