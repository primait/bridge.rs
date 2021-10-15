#[cfg(not(feature = "auth0"))]
mod common;

#[cfg(all(not(feature = "blocking"), feature = "auth0"))]
mod async_auth0;
#[cfg(all(not(feature = "blocking"), not(feature = "auth0")))]
mod async_bridge;
#[cfg(all(feature = "blocking", not(feature = "auth0")))]
mod blocking;
