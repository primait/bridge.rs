use std::ops::RangeInclusive;
use std::time::Duration;

use reqwest::Url;

#[derive(Clone)]
pub struct Config {
    /// Auth0 base url.
    /// This is the url used by the bridge to fetch new tokens
    pub token_url: Url,
    /// The microservice implementing this Bridge
    pub caller: String,
    /// The microservice I want to connect to.
    /// This should match the string defined in auth0 as "audience"
    pub audience: String,
    /// Cache configuration. Could be a redis connection string or inmemory cache
    pub cache_type: CacheType,
    /// The key to use in order to encrypt `CachedToken` in redis
    pub token_encryption_key: String,
    /// Every {check_interval} the `TokenDispenser` actor checks if token needs to be refreshed
    pub check_interval: Duration,
    /// this is a time period in which the token should be considered stale. Expressed as a range that represent the whole lifespan of the token.
    /// The lower bound means that from that from that moment onward the library will try to refresh the token at the scheduled time
    /// the higher bound means that the library will try to refresh the token as soon as possible
    pub staleness_check_percentage: StalenessCheckPercentage,
    /// Auth0 client identifier. Every machine should share the same identifier
    pub client_id: String,
    /// Auth0 client secret. Every machine should share the same secret
    pub client_secret: String,
    /// JWKS url
    /// This is the url that the bridge uses to fetch JWKS
    pub jwks_url: Url,
}

impl Config {
    pub fn token_url(&self) -> &Url {
        &self.token_url
    }

    pub fn caller(&self) -> &str {
        &self.caller
    }

    pub fn audience(&self) -> &str {
        &self.audience
    }

    pub fn cache_type(&self) -> &CacheType {
        &self.cache_type
    }

    pub fn token_encryption_key(&self) -> &str {
        &self.token_encryption_key
    }

    pub fn check_interval(&self) -> &Duration {
        &self.check_interval
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

    pub fn staleness_check_percentage(&self) -> &StalenessCheckPercentage {
        &self.staleness_check_percentage
    }

    pub fn is_inmemory_cache(&self) -> bool {
        self.cache_type == CacheType::Inmemory
    }

    #[cfg(test)]
    pub fn test_config() -> Config {
        use std::str::FromStr;
        Config {
            token_url: Url::from_str(&format!("{}/{}", mockito::server_url().as_str(), "token")).unwrap(),
            jwks_url: Url::from_str(&format!("{}/{}", mockito::server_url().as_str(), "jwks")).unwrap(),
            caller: "caller".to_string(),
            audience: "audience".to_string(),
            cache_type: CacheType::Inmemory,
            token_encryption_key: "32char_long_token_encryption_key".to_string(),
            check_interval: Duration::from_secs(10),
            staleness_check_percentage: StalenessCheckPercentage::default(),
            client_id: "client_id".to_string(),
            client_secret: "client_secret".to_string(),
        }
    }
}

// Eg. `Redis("redis://{host}:{port}?{ParamKey1}={ParamKey2}")` or `Inmemory` for inmemory cache
#[derive(Clone, PartialEq)]
pub enum CacheType {
    Redis(String),
    Inmemory,
}

impl CacheType {
    pub fn redis_connection_url(&self) -> &str {
        match &self {
            CacheType::Redis(url) => url,
            CacheType::Inmemory => {
                panic!("Something went wrong getting Redis connection string")
            }
        }
    }
}

#[derive(Clone)]
pub struct StalenessCheckPercentage(RangeInclusive<f64>);

impl StalenessCheckPercentage {
    pub fn new(min: f64, max: f64) -> Self {
        assert!((0.0..=1.0).contains(&min));
        assert!((0.0..=1.0).contains(&max));
        assert!(min <= max);

        Self(min..=max)
    }

    pub fn random_value_between(&self) -> f64 {
        use rand::Rng;
        rand::thread_rng().gen_range(self.0.clone())
    }
}

impl Default for StalenessCheckPercentage {
    fn default() -> Self {
        Self(0.6..=0.9)
    }
}

impl From<RangeInclusive<f64>> for StalenessCheckPercentage {
    fn from(range: RangeInclusive<f64>) -> Self {
        Self::new(*range.start(), *range.end())
    }
}
