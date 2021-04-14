use reqwest::Url;
use std::time::Duration;

pub struct Auth0Configuration {
    base_url: Url,
    audience: String,
    //cache: Auth0ConfigurationCache,
    //client: Auth0ConfigurationClient,
}

impl Auth0Configuration {
    pub fn new(
        base_url: Url,
        audience: String,
        //cache: Auth0ConfigurationCache,
        //client: Auth0ConfigurationClient,
    ) -> Self {
        Self {
            base_url,
            audience,
            //cache,
            //client,
        }
    }
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
    pub fn audience(&self) -> String {
        self.audience.clone()
    }
    pub fn redis_connection_uri(&self) -> String {
        //self.cache.redis_connection_uri.clone()

        "".to_string()
    }
}

pub struct Auth0ConfigurationCache {
    redis_connection_uri: String,
    namespace: String,
    encryption_key: String,
}

impl Auth0ConfigurationCache {
    fn new(redis_connection_uri: String, namespace: String, encryption_key: String) -> Self {
        Self {
            redis_connection_uri,
            namespace,
            encryption_key,
        }
    }
}

pub struct Auth0ConfigurationClient {
    check_interval: Duration,
    min_token_duration: f32,
    max_token_duration: f32,
    client_id: String,
    client_secret: String,
}

impl Auth0ConfigurationClient {
    fn new(client_id: String, client_secret: String) -> Self {
        Self {
            check_interval: Duration::from_secs(1),
            min_token_duration: 0.5,
            max_token_duration: 0.5,
            client_id,
            client_secret,
        }
    }
}
