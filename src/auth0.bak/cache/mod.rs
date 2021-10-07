use redis::{Client, Commands};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::auth0::keyset::JsonWebKeySet;
use crate::auth0::token::Token;
use crate::errors::PrimaBridgeResult;

mod crypto;

#[derive(Clone, Debug)]
pub struct Redis {
    client: Client,
    encryption_key: String,
    caller: String,
    audience: String,
}

impl Redis {
    pub fn new(config: &crate::auth0::config::Config) -> PrimaBridgeResult<Self> {
        Ok(Self {
            client: redis::Client::open(config.redis_connection_uri())?,
            encryption_key: config.token_encryption_key().to_string(),
            caller: config.caller().to_string(),
            audience: config.audience().to_string(),
        })
    }

    fn get<'a, T: DeserializeOwned>(&self, key: &str) -> PrimaBridgeResult<Option<T>> {
        self.client
            .get_connection()?
            .get::<_, Option<Vec<u8>>>(key)?
            .map(|value| crypto::decrypt(self.encryption_key.as_str(), value))
            .transpose()
    }

    fn set<T: Serialize>(&mut self, key: &str, val: &T) -> PrimaBridgeResult<()> {
        let mut connection: redis::Connection = self.client.get_connection()?;
        let encrypted_value: Vec<u8> = crypto::encrypt(val, self.encryption_key.as_str())?;
        connection.set(key, encrypted_value)?;
        Ok(())
    }

    fn set_ex<T: Serialize>(&mut self, key: &str, val: &T, exp: usize) -> PrimaBridgeResult<()> {
        let mut connection: redis::Connection = self.client.get_connection()?;
        let encrypted_value: Vec<u8> = crypto::encrypt(val, self.encryption_key.as_str())?;
        connection.set_ex(key, encrypted_value, exp)?;
        Ok(())
    }

    pub fn get_token(&self) -> PrimaBridgeResult<Option<Token>> {
        self.get::<Token>(&self.token_key())
    }

    pub fn set_token(&mut self, token: &Token) -> PrimaBridgeResult<()> {
        self.set_ex(&self.token_key(), token, token.lifetime_in_seconds())
    }

    pub fn get_jwks(&self) -> PrimaBridgeResult<Option<JsonWebKeySet>> {
        self.get::<JsonWebKeySet>(&self.jwks_key())
    }

    pub fn set_jwks(&mut self, json_web_keys_set: &JsonWebKeySet) -> PrimaBridgeResult<()> {
        self.set(&self.jwks_key(), json_web_keys_set)
    }

    fn token_key(&self) -> String {
        format!("auth0rs_tokens:{}:{}", &self.caller, &self.audience)
    }

    fn jwks_key(&self) -> String {
        format!("auth0rs_jwks:{}:{}", &self.caller, &self.audience)
    }
}

#[cfg(all(test, feature = "auth0"))]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::auth0::cache::CacheEntry;
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
