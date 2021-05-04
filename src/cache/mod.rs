use aes::Aes256 as Aes256Alg;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth0_config::Auth0Config;
#[cfg(feature = "inmemory")]
pub use crate::cache::inmemory::InMemoryCache as CacheImpl;
#[cfg(not(feature = "inmemory"))]
pub use crate::cache::redis_cache::RedisCache as CacheImpl;
use crate::errors::PrimaBridgeResult;

mod inmemory;
mod redis_cache;

const IV: &str = "301a9e39735f4646";

type Aes256 = Cbc<Aes256Alg, Pkcs7>;

pub trait Cache {
    fn new(config: &Auth0Config) -> PrimaBridgeResult<Self>
    where
        Self: Sized;

    fn get(&mut self, key: &str) -> PrimaBridgeResult<Option<CacheEntry>>;
    fn set(&mut self, key: &str, value: CacheEntry) -> PrimaBridgeResult<()>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheEntry {
    token: String,
    issue_date: DateTime<Utc>,
    expire_date: DateTime<Utc>,
}

impl CacheEntry {
    pub fn new(token: String, issue_date: DateTime<Utc>, expire_date: DateTime<Utc>) -> Self {
        Self {
            token,
            issue_date,
            expire_date,
        }
    }

    pub fn encrypt(&self, token_encryption_key: &str) -> PrimaBridgeResult<Vec<u8>> {
        let json: String = serde_json::to_string(&self)?;
        // `unwrap` here is fine because `IV` is set here and the only error returned is: `InvalidKeyIvLength`
        // and this must never happen
        let cipher: Aes256 =
            Aes256::new_var(&token_encryption_key.as_bytes(), IV.as_bytes()).unwrap();
        Ok(cipher.encrypt_vec(json.as_bytes()))
    }

    pub fn decrypt(token_encryption_key: &str, encrypted: Vec<u8>) -> PrimaBridgeResult<Self> {
        // `unwrap` here is fine because `IV` is set here and the only error returned is: `InvalidKeyIvLength`
        // and this must never happen
        let cipher: Aes256 =
            Aes256::new_var(&token_encryption_key.as_bytes(), IV.as_bytes()).unwrap();
        let decrypted: Vec<u8> = cipher.decrypt_vec(encrypted.as_slice())?;
        Ok(serde_json::from_str::<Self>(
            String::from_utf8(decrypted)?.as_str(),
        )?)
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn issue_date(&self) -> &DateTime<Utc> {
        &self.issue_date
    }

    pub fn expire_date(&self) -> &DateTime<Utc> {
        &self.expire_date
    }

    // Return the percentage of the remaining life
    pub fn remaining_life_percentage(&self) -> f64 {
        dbg!(*&self.expire_date.timestamp_millis());
        dbg!(Utc::now().timestamp_millis());
        if *&self.expire_date.timestamp_millis() - Utc::now().timestamp_millis() <= 0 {
            0.0
        } else {
            let expire_millis: i64 = *&self.expire_date.timestamp_millis();
            let issue_millis: i64 = *&self.issue_date.timestamp_millis();
            let remaining: f64 = (expire_millis - Utc::now().timestamp_millis()) as f64;
            let total: f64 = (expire_millis - issue_millis) as f64;
            remaining / total * 100.0
        }
    }
}

#[cfg(all(test, feature = "auth0"))]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::cache::CacheEntry;
    use crate::errors::PrimaBridgeResult;

    #[test]
    fn decrypt_and_encrypt_token() {
        let key_1: &str = "needalengthof32inordertoencrypt!";
        let token: String = "token".to_string();
        let issue_date: DateTime<Utc> = Utc::now();
        let expire_date: DateTime<Utc> = Utc::now();
        let entry: CacheEntry = CacheEntry::new(token, issue_date, expire_date);

        // Encrypt and decrypt is successful
        let encrypted: Vec<u8> = entry.encrypt(key_1).unwrap();
        let decrypted: CacheEntry = CacheEntry::decrypt(key_1, encrypted.clone()).unwrap();
        assert_eq!(entry.token, decrypted.token);

        // Decrypt with different key result in an error
        let key_2: &str = "wronglengthof32inordertoencrypt!";
        let decrypted_result: PrimaBridgeResult<CacheEntry> = CacheEntry::decrypt(key_2, encrypted);

        assert_ne!(key_1, key_2);
        assert!(decrypted_result.is_err());
    }
}