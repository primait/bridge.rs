use aes::Aes256 as Aes256Alg;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::errors::PrimaBridgeResult;

type Aes256 = Cbc<Aes256Alg, Pkcs7>;

const IV: &str = "301a9e39735f4646";

pub fn encrypt<T: Serialize>(value: &T, token_encryption_key: &str) -> PrimaBridgeResult<Vec<u8>> {
    let json: String = serde_json::to_string(value)?;
    // `unwrap` here is fine because `IV` is set here and the only error returned is: `InvalidKeyIvLength`
    // and this must never happen
    let cipher: Aes256 = Aes256::new_var(&token_encryption_key.as_bytes(), IV.as_bytes()).unwrap();
    Ok(cipher.encrypt_vec(json.as_bytes()))
}

pub fn decrypt<T: DeserializeOwned>(
    token_encryption_key: &str,
    encrypted: Vec<u8>,
) -> PrimaBridgeResult<T> {
    // `unwrap` here is fine because `IV` is set here and the only error returned is: `InvalidKeyIvLength`
    // and this must never happen
    let cipher: Aes256 = Aes256::new_var(&token_encryption_key.as_bytes(), IV.as_bytes()).unwrap();
    let decrypted: Vec<u8> = cipher.decrypt_vec(encrypted.as_slice())?;
    Ok(serde_json::from_slice(decrypted.as_slice())?)
}
