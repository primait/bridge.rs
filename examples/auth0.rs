use mockito::{mock, Mock};
use prima_bridge::auth0_config::Auth0Config;
use prima_bridge::Bridge;
use rand::Rng;
use reqwest::Url;
use std::time::Duration;

#[tokio::main]
#[cfg(not(feature = "blocking"))]
async fn main() {
    let _m = mock("GET", "/").with_status(200).with_body("OK").create();
    let _auth0 = auth0_mock();
    let url: Url = Url::parse(mockito::server_url().as_str()).unwrap();
    let conf: Auth0Config = new_auth0_config();
    let bridge: Bridge = Bridge::new(url, conf)
        .await
        .expect("Cannot create new bridge");

    let mut i = 0;
    loop {
        i += 1;
        dbg!(i);
        dbg!(bridge.get_headers().await);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

#[cfg(feature = "blocking")]
fn main() {
    let _m = mock("GET", "/").with_status(200).with_body("OK").create();
    let _auth0 = auth0_mock();
    let url: Url = Url::parse(mockito::server_url().as_str()).unwrap();
    let conf: Auth0Config = new_auth0_config();
    let bridge: Bridge = Bridge::new(url, conf).expect("Cannot create new bridge");

    let mut i = 0;
    loop {
        i += 1;
        dbg!(i);
        dbg!(bridge.get_headers());
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn auth0_mock() -> Mock {
    mock("GET", "/token")
        .with_status(200)
        .with_body_from_fn(move |w| {
            w.write_all({
                let token = rand::thread_rng()
                    .sample_iter(rand::distributions::Alphanumeric)
                    .take(10)
                    .map(char::from)
                    .collect::<String>();
                format!("{{\"token\": \"{}\"}}", token).as_bytes()
            })
        })
        .create()
}

fn new_auth0_config() -> Auth0Config {
    let auth0_url: Url =
        Url::parse(&format!("{}/{}", mockito::server_url().as_str(), "token")).unwrap();

    Auth0Config::new(
        auth0_url,
        "test".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        Duration::from_secs(1),
        0,
        100,
        "".to_string(),
        "".to_string(),
    )
}
