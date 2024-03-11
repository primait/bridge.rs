use aes::cipher::block_padding::UnpadError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Auth0Error {
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error("received bad status code while getting token: {0}")]
    JwtFetchAuthError(u16),
    #[error("failed to fetch jwt from {0}. Status code: {0}; error: {1}")]
    JwtFetchError(u16, String, reqwest::Error),
    #[error("failed to deserialize jwt from {0}. {1}")]
    JwtFetchDeserializationError(String, reqwest::Error),
    #[error(transparent)]
    JwksClientError(#[from] jwks_client_rs::JwksClientError),
    #[error("failed to fetch jwt from {0}. Status code: {0}; error: {1}")]
    JwksHttpError(String, reqwest::Error),
    #[error("redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error(transparent)]
    CryptoError(#[from] UnpadError),
}
