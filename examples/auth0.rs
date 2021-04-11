use mockito::mock;
use prima_bridge::Bridge;
use rand::Rng;
use reqwest::Url;

#[tokio::main]
async fn main() {
    let _m = mock("GET", "/").with_status(200).with_body("OK").create();

    let _auth0 = mock("GET", "/token")
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
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let auth0_url = Url::parse(&format!("{}/{}", mockito::server_url().as_str(), "token")).unwrap();

    let mut bridge = Bridge::new(url);
    bridge.with_auth0_authentication(auth0_url, "test").await;

    let mut i = 0;
    loop {
        i += 1;
        dbg!(i);
        dbg!(bridge.get_headers().await);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
