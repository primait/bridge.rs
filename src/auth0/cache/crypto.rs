use chacha20poly1305::{aead::Aead, AeadCore, KeyInit, XChaCha20Poly1305};
use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::auth0::errors::Auth0Error;

const NONCE_SIZE: usize = 24;

pub fn encrypt<T: Serialize>(value_ref: &T, token_encryption_key_str: &str) -> Result<Vec<u8>, Auth0Error> {
    let json: String = serde_json::to_string(value_ref)?;

    let enc = XChaCha20Poly1305::new_from_slice(token_encryption_key_str.as_bytes()).unwrap();
    let nonce = XChaCha20Poly1305::generate_nonce(&mut thread_rng());

    let mut ciphertext = enc.encrypt(&nonce, json.as_bytes())?;
    ciphertext.extend(nonce);

    Ok(ciphertext)
}

pub fn decrypt<T>(token_encryption_key_str: &str, encrypted: &[u8]) -> Result<T, Auth0Error>
where
    for<'de> T: Deserialize<'de>,
{
    let dec = XChaCha20Poly1305::new_from_slice(token_encryption_key_str.as_bytes()).unwrap();
    
    let ciphertext = encrypted.get(..encrypted.len()-NONCE_SIZE);
    let nonce = encrypted.get(encrypted.len()-NONCE_SIZE..);

    let (Some(ciphertext), Some(nonce)) = (ciphertext, nonce) else {
        return Err(Auth0Error::CryptoError(chacha20poly1305::Error));
    };

    let nonce = chacha20poly1305::XNonce::from_slice(nonce);
    let plaintext = dec.decrypt(nonce, ciphertext)?;
    Ok(serde_json::from_slice(&plaintext)?)
}
