use std::time::Duration;

use reqwest::Url;

pub struct Auth0Config {
    /// Auth0 base url.
    /// This is the url used by the bridge to fetch new tokens
    base_url: Url,
    /// The microservice implementing this Bridge
    caller: String,
    /// The microservice I want to connect to. Eg. crash, squillo etc.
    audience: String,
    /// Redis connection string. Eg. redis://{host}:{port}?{ParamKey1}={ParamKey2}
    redis_connection_uri: String,
    /// Who am I. The microservice that is instantiating this client
    token_namespace: String,
    /// The key to use in order to encrypt `CachedToken` in redis
    token_encryption_key: String,
    /// Every {check_interval} the `TokenDispenser` actor checks if token needs to be refreshed
    check_interval: Duration,
    /// todo: explain. todo2: could be useful to cast to something like a Percentage struct. It's good
    /// todo: if 0 < x < 100
    min_token_remaining_life_percentage: i32,
    /// todo: explain
    max_token_remaining_life_percentage: i32,
    /// Auth0 client identifier. Every machine should share the same identifier
    client_id: String,
    /// Auth0 client secret. Every machine should share the same secret
    client_secret: String,
    /// JWKS url
    /// This is that the bridge uses to fetch JWKS
    jwks_url: Url,
}

impl Auth0Config {
    pub fn new(
        base_url: Url,
        jwks_url: Url,
        caller: String,
        audience: String,
        redis_connection_uri: String,
        token_namespace: String,
        token_encryption_key: String,
        check_interval: Duration,
        min_token_remaining_life_percentage: i32,
        max_token_remaining_life_percentage: i32,
        client_id: String,
        client_secret: String,
    ) -> Self {
        // todo: fail if `token_encryption_key.len() != 32`
        Self {
            base_url,
            caller,
            audience,
            redis_connection_uri,
            token_namespace,
            token_encryption_key,
            check_interval,
            min_token_remaining_life_percentage,
            max_token_remaining_life_percentage,
            client_id,
            client_secret,
            jwks_url,
        }
    }
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn caller(&self) -> &str {
        &self.caller
    }

    pub fn audience(&self) -> &str {
        &self.audience
    }

    pub fn redis_connection_uri(&self) -> &str {
        &self.redis_connection_uri
    }

    pub fn token_namespace(&self) -> &str {
        &self.token_namespace
    }

    pub fn token_encryption_key(&self) -> &str {
        &self.token_encryption_key
    }

    pub fn check_interval(&self) -> &Duration {
        &self.check_interval
    }

    pub fn min_token_remaining_life_percentage(&self) -> i32 {
        self.min_token_remaining_life_percentage
    }

    pub fn max_token_remaining_life_percentage(&self) -> i32 {
        self.max_token_remaining_life_percentage
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }

    pub fn jwks_url(&self) -> &Url {
        &self.jwks_url
    }
}
