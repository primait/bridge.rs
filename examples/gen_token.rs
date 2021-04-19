use alcoholic_jwt::{token_kid, validate, Validation, JWKS};
use reqwest;
use reqwest::blocking::Client;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    client_id: String,
    client_secret: String,
    audience: String,
    domain: String,
}

#[derive(Serialize)]
pub struct FetchTokenRequest {
    client_id: String,
    client_secret: String,
    audience: String,
    grant_type: String,
}

#[derive(Deserialize, Debug)]
pub struct FetchTokenResponse {
    access_token: String,
    expires_in: i32,
    token_type: String,
}

fn main() {
    let opt = Opt::from_args();
    let client = Client::new();
    print!("fetching jwks...");
    let jwks = fetch_jwks(&client, &opt.domain);
    println!("OK");
    println!();
    let url = Url::parse(&format!("{}/oauth/token", opt.domain)).unwrap();
    let response: FetchTokenResponse = client
        .post(url)
        .json(&FetchTokenRequest {
            client_id: opt.client_id,
            client_secret: opt.client_secret,
            audience: opt.audience,
            grant_type: "client_credentials".to_string(),
        })
        .send()
        .expect("error fetching token")
        .json()
        .expect("error deserializing response");

    println!("Token: {}", &response.access_token);
    println!("Expires in: {}", &response.expires_in);
    println!();
    println!("Token validation");
    let start = std::time::Instant::now();
    for i in 1..=(response.expires_in + 5) {
        std::thread::sleep(std::time::Duration::from_secs(1));
        print!("{}. ({} ms elapsed) ", i, start.elapsed().as_millis());
        validate_token(&response.access_token, &jwks);
    }
}

fn fetch_jwks(client: &Client, domain: &str) -> JWKS {
    let url = Url::parse(&format!("{}/.well-known/jwks.json", domain)).unwrap();
    client.get(url).send().unwrap().json().unwrap()
}

fn validate_token(token: &str, jwks: &JWKS) {
    let validations = vec![
        Validation::Issuer("https://matteosister.eu.auth0.com/".into()),
        Validation::SubjectPresent,
        Validation::NotExpired,
    ];
    let kid = token_kid(token)
        .expect("Failed to decode token headers")
        .expect("No 'kid' claim present in token");
    let jwk = jwks.find(&kid).expect("Specified key not found in set");
    let result = validate(token, jwk, validations);
    match result {
        Ok(_) => {
            println!("the token is valid")
        }
        Err(e) => {
            println!("the token is NOT valid");
            dbg!(e);
        }
    }
}
