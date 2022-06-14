use aes::{
    cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit},
    Aes256,
};
use cbc::{Decryptor, Encryptor};
use serde::{Deserialize, Serialize};

use crate::auth0::errors::Auth0Error;

type Aes256Enc = Encryptor<Aes256>;
type Aes256Dec = Decryptor<Aes256>;

const IV: &str = "301a9e39735f4646";

pub fn encrypt<T: Serialize>(value_ref: &T, token_encryption_key_str: &str) -> Result<Vec<u8>, Auth0Error> {
    let json: String = serde_json::to_string(value_ref)?;

    // `unwrap` here is fine because `IV` is set here and the only error returned is: `InvalidKeyIvLength`
    // and this must never happen
    let mut buf = vec![0u8; json.len()];
    buf[..json.len()].copy_from_slice(json.as_bytes());
    let ct = Aes256Enc::new(token_encryption_key_str.as_bytes().into(), IV.as_bytes().into())
        .encrypt_padded_mut::<Pkcs7>(&mut buf, json.len())
        .unwrap();

    Ok(ct.to_vec())
}

pub fn decrypt<T>(token_encryption_key_str: &str, encrypted: &[u8]) -> Result<T, Auth0Error>
where
    for<'de> T: Deserialize<'de>,
{
    // `unwrap` here is fine because `IV` is set here and the only error returned is: `InvalidKeyIvLength`
    // and this must never happen
    let mut buf = Vec::from(encrypted);
    let pt = Aes256Dec::new(token_encryption_key_str.as_bytes().into(), IV.as_bytes().into())
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .unwrap();

    Ok(serde_json::from_slice(pt)?)
}
