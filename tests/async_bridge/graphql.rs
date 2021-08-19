use std::error::Error;

use mockito;
use mockito::{mock, Matcher, Mock};
use reqwest::Url;
use serde::Deserialize;
use serde_json::json;

use prima_bridge::prelude::*;
//use prima_bridge::Request;
use reqwest::header::{HeaderName, HeaderValue};
use std::fs;

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

#[tokio::test]
async fn request_with_custom_headers() -> Result<(), Box<dyn Error>> {
    let query = "query { hello }";
    let (_m, bridge) = create_gql_bridge(
        200,
        query,
        "{\"data\": {\"person\": {\"name\": \"Pippo\"}}}",
    );

    let variables: Option<String> = None;
    let response = GraphQLRequest::new(&bridge, (query, variables))?
        .with_custom_headers(vec![(
            HeaderName::from_static("x-prima"),
            HeaderValue::from_static("test-value"),
        )])
        .send()
        .await?;

    assert!(response.is_ok());

    Ok(())
}

#[derive(Deserialize, Debug)]
struct GqlResponse {
    hero: Hero,
}

#[derive(Deserialize, Debug)]
struct Hero {
    name: String,
    #[serde(rename = "heroFriends")]
    friends: Vec<Option<Friend>>,
}

#[derive(Deserialize, Debug)]
struct Friend {
    id: String,
    name: Option<String>,
}

#[tokio::test]
async fn error_response_parser() -> Result<(), Box<dyn Error>> {
    let query = file_content("graphql/hero.graphql");
    let (_m, bridge) = create_gql_bridge(
        200,
        query.as_str(),
        file_content("graphql/error_with_data.json").as_str(),
    );
    let variables: Option<String> = None;
    let response = GraphQLRequest::new(&bridge, (query.as_str(), variables))?
        .send()
        .await?;
    let parsed_response = response.get_graphql_response::<GqlResponse>()?;

    assert!(!parsed_response.is_ok());
    assert!(parsed_response.has_parsed_data());
    assert_eq!(1, parsed_response.get_errors().len());

    Ok(())
}

#[tokio::test]
async fn error_response_parser_with_non_null_element() -> Result<(), Box<dyn Error>> {
    let query = file_content("graphql/hero.graphql");
    let (_m, bridge) = create_gql_bridge(
        200,
        query.as_str(),
        file_content("graphql/error_non_null_response.json").as_str(),
    );
    let variables: Option<String> = None;
    let response = GraphQLRequest::new(&bridge, (query.as_str(), variables))?
        .send()
        .await?;
    let parsed_response = response.get_graphql_response::<GqlResponse>()?;

    assert!(!parsed_response.is_ok());
    assert!(parsed_response.has_parsed_data());
    assert_eq!(1, parsed_response.get_errors().len());

    Ok(())
}

#[tokio::test]
async fn error_response_parser_with_error() -> Result<(), Box<dyn Error>> {
    let query = file_content("graphql/hero.graphql");
    let (_m, bridge) = create_gql_bridge(
        200,
        query.as_str(),
        file_content("graphql/error.json").as_str(),
    );
    let variables: Option<String> = None;
    let response = GraphQLRequest::new(&bridge, (query.as_str(), variables))?
        .send()
        .await?;
    let parsed_response = response.get_graphql_response::<GqlResponse>()?;

    assert!(!parsed_response.is_ok());
    assert!(!parsed_response.has_parsed_data());
    assert_eq!(1, parsed_response.get_errors().len());

    Ok(())
}

fn create_gql_bridge(status_code: usize, query: &str, body: &str) -> (Mock, Bridge) {
    let mock = mock("POST", "/")
        .match_header("content-type", "application/json")
        .match_body(Matcher::Json(json!({ "query": query })))
        .with_status(status_code)
        .with_body(body)
        .create();

    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge = Bridge::builder().build(url);

    (mock, bridge)
}

fn file_content(path_relative_to_recourses: &str) -> String {
    let mut base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_dir.push("tests/resources");
    fs::read_to_string(format!(
        "{}/{}",
        base_dir.to_str().unwrap(),
        path_relative_to_recourses
    ))
    .unwrap()
}
