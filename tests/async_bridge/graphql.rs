use std::error::Error;

use mockito;
use mockito::{mock, Matcher, Mock};
use reqwest::Url;
use serde::Deserialize;
use serde_json::json;

use prima_bridge::prelude::*;
use prima_bridge::Request;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Person {
    name: String,
}

#[tokio::test]
async fn simple_request() -> Result<(), Box<dyn Error>> {
    let query = "query { hello }";
    let (_m, bridge) = create_gql_bridge(
        200,
        query,
        "{\"data\": {\"person\": {\"name\": \"Pippo\"}}}",
    );
    let variables: Option<String> = None;

    let result: Person = Request::graphql(&bridge, (query, variables))?
        .send()
        .await?
        .get_data(&["person"])?;

    assert_eq!(
        Person {
            name: "Pippo".to_string()
        },
        result
    );

    Ok(())
}

fn create_gql_bridge(status_code: usize, query: &str, body: &str) -> (Mock, Bridge) {
    let mock = mock("POST", "/")
        .match_body(Matcher::Json(json!({ "query": query })))
        .with_status(status_code)
        .with_body(body)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::new(url);

    (mock, bridge)
}
