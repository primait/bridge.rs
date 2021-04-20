use serde::Deserialize;

mod jwks;

#[cfg(not(feature = "blocking"))]
pub use async_impl::TokenDispenserHandle;
#[cfg(feature = "blocking")]
pub use blocking::TokenDispenserHandle;

#[cfg(not(feature = "blocking"))]
mod async_impl;
#[cfg(feature = "blocking")]
mod blocking;

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    token: String,
}
