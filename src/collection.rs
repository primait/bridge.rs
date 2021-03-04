use crate::Bridge;
use once_cell::sync::{Lazy, OnceCell};
use reqwest::Url;
use std::collections::HashMap;
use std::sync::RwLock;

//static BRIDGES: Lazy<RwLock<HashMap<Url, Bridge>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub struct BridgesCollection(HashMap<Url, Bridge>);

static BRIDGES: OnceCell<RwLock<BridgesCollection>> = OnceCell::new();

impl BridgesCollection {
    pub fn add(endpoint: Url) {
        let bridge = Bridge {
            #[cfg(feature = "blocking")]
            client: reqwest::blocking::Client::new(),
            #[cfg(not(feature = "blocking"))]
            client: reqwest::Client::new(),
            endpoint: endpoint.clone(),
        };
        BRIDGES
            .get_or_init(|| RwLock::new(BridgesCollection(HashMap::default())))
            .write()
            .unwrap()
            .0
            .insert(endpoint, bridge);
    }

    pub fn get(endpoint: Url) -> &'static Bridge {
        let bridges = *BRIDGES
            .get()
            .expect("Bridges collection is not initialized")
            .read()
            .unwrap();
        bridges.0.get(&endpoint).unwrap()
    }
}
