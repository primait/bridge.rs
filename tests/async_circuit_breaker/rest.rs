use std::error::Error;

use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::json;

use prima_bridge::prelude::*;

use crate::common::*;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
struct Data {
    hello: String,
}

#[tokio::test]
async fn simple_request() -> Result<(), Box<dyn Error>> {
    let (_m, bridge) = create_bridge(200, "{\"hello\": \"world!\"}");
    let result: String = RestRequest::new(&bridge)
        .send()
        .await?
        .get_data(&["hello"])?;

    assert_eq!("world!", result.as_str());

    Ok(())
}
