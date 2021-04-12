use serde::Deserialize;

#[cfg(not(feature = "blocking"))]
mod async_impl;
#[cfg(not(feature = "blocking"))]
pub use async_impl::TokenDispenserHandle;

#[cfg(feature = "blocking")]
mod blocking;
#[cfg(feature = "blocking")]
pub use blocking::TokenDispenserHandle;

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    token: String,
}
