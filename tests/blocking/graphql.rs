use std::error::Error;

use mockito;
use mockito::{mock, Matcher, Mock};
use reqwest::Url;
use serde::Deserialize;
use serde_json::json;

use prima_bridge::prelude::*;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Person {
    name: String,
}

#[test]
fn simple_request_ok() -> Result<(), Box<dyn Error>> {
    let query = "query { hello }";
    let (_m, bridge) = create_gql_bridge(
        200,
        query,
        "{\"data\": {\"person\": {\"name\": \"Pippo\"}}}",
    );
    let variables: Option<String> = None;

    let result: Person = GraphQLRequest::new(&bridge, (query, variables))?
        .send()?
        .get_data(&["person"])?;

    assert_eq!(
        Person {
            name: "Pippo".to_string()
        },
        result
    );

    Ok(())
}

#[test]
fn simple_request_ignoring_status_code() -> Result<(), Box<dyn Error>> {
    let query = "query { hello }";
    let (_m, bridge) = create_gql_bridge(
        400,
        query,
        "{\"data\": {\"person\": {\"name\": \"Pippo\"}}}",
    );
    let variables: Option<String> = None;

    let result: Person = bridge
        .request(RequestType::graphql(query, variables))
        .ignore_status_code()
        .send()?
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
