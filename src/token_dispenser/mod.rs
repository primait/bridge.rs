use serde::{Deserialize, Serialize};

#[cfg(not(feature = "blocking"))]
pub use async_impl::TokenDispenserHandle;
#[cfg(feature = "blocking")]
pub use blocking::TokenDispenserHandle;

#[cfg(not(feature = "blocking"))]
mod async_impl;
#[cfg(feature = "blocking")]
mod blocking;

#[derive(Serialize, Debug)]
pub struct TokenRequest {
    client_id: String,
    client_secret: String,
    audience: String,
    grant_type: String,
}

impl TokenRequest {
    pub fn new(client_id: String, client_secret: String, audience: String) -> Self {
        Self {
            client_id,
            client_secret,
            audience,
            grant_type: "client_credentials".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TokenResponse {
    access_token: String,
    scope: String,
    expires_in: i32,
    token_type: String,
}

fn random(x: f64, y: f64) -> f64 {
    use rand::Rng;
    match x - y {
        z if z == 0.0 => x,
        z if z > 0.0 => rand::thread_rng().gen_range(y..=x),
        _ => rand::thread_rng().gen_range(x..=y),
    }
}
