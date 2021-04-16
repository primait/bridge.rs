#[cfg(not(feature = "auth0"))]
fn main() {
    println!("Run with `auth0` feature enabled to run this example")
}

#[tokio::main]
#[cfg(all(not(feature = "blocking"), feature = "auth0"))]
async fn main() {
    let _m = mockito::mock("GET", "/")
        .with_status(200)
        .with_body("OK")
        .create();
    let _auth0 = auth0_mock();
    let url: reqwest::Url = reqwest::Url::parse(mockito::server_url().as_str()).unwrap();
    let conf: prima_bridge::auth0_config::Auth0Config = new_auth0_config();
    let bridge: prima_bridge::Bridge = prima_bridge::Bridge::new(url, conf)
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

#[cfg(all(feature = "blocking", feature = "auth0"))]
fn main() {
    let _m = mockito::mock("GET", "/")
        .with_status(200)
        .with_body("OK")
        .create();
    let _auth0 = auth0_mock();
    let url: reqwest::Url = reqwest::Url::parse(mockito::server_url().as_str()).unwrap();
    let conf: prima_bridge::auth0_config::Auth0Config = new_auth0_config();
    let bridge: prima_bridge::Bridge =
        prima_bridge::Bridge::new(url, conf).expect("Cannot create new bridge");

    let mut i = 0;
    loop {
        i += 1;
        dbg!(i);
        dbg!(bridge.get_headers());
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

#[cfg(feature = "auth0")]
fn auth0_mock() -> mockito::Mock {
    use rand::Rng;
    mockito::mock("GET", "/token")
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

#[cfg(feature = "auth0")]
fn new_auth0_config() -> prima_bridge::auth0_config::Auth0Config {
    let auth0_url: reqwest::Url =
        reqwest::Url::parse(&format!("{}/{}", mockito::server_url().as_str(), "token")).unwrap();

    prima_bridge::auth0_config::Auth0Config::new(
        auth0_url,
        "test".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        std::time::Duration::from_secs(1),
        0,
        100,
        "".to_string(),
        "".to_string(),
    )
}
