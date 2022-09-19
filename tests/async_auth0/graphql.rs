use std::error::Error;

use mockito::{mock, Matcher, Mock};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Url;
use serde::Deserialize;
use serde_json::json;

use prima_bridge::prelude::*;
use prima_bridge::ParsedGraphqlResponseExt;

use crate::async_auth0::{config, Auth0Mocks};

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Person {
    name: String,
}

#[tokio::test]
async fn simple_request() -> Result<(), Box<dyn Error>> {
    let query = "query { hello }";
    let (_m, bridge) = create_gql_bridge(200, query, "{\"data\": {\"person\": {\"name\": \"Pippo\"}}}").await;

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
    let (_m, bridge) = create_gql_bridge(200, query, "{\"data\": {\"person\": {\"name\": \"Pippo\"}}}").await;

    let variables: Option<String> = None;
    let gql_request: GraphQLRequest = GraphQLRequest::new(&bridge, (query, variables))?.with_custom_headers(vec![(
        HeaderName::from_static("x-prima"),
        HeaderValue::from_static("test-value"),
    )]);

    let bearer: String = bridge.token().unwrap().to_bearer();
    let headers = gql_request.get_all_headers();
    let header_value: Option<&HeaderValue> = headers.get(reqwest::header::AUTHORIZATION.as_str());
    assert!(bearer.starts_with("Bearer "));
    assert_eq!(header_value.unwrap().to_str().unwrap(), bearer);

    let response = gql_request.send().await?;
    assert!(response.is_ok());
    Ok(())
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct GqlResponse {
    hero: Hero,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Hero {
    name: String,
    #[serde(rename = "heroFriends")]
    friends: Vec<Option<Friend>>,
}

#[allow(dead_code)]
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
    )
    .await;
    let variables: Option<String> = None;
    let response = GraphQLRequest::new(&bridge, (query.as_str(), variables))?
        .send()
        .await?;
    let parsed_response = response.parse_graphql_response::<GqlResponse>()?;

    assert!(parsed_response.is_err());
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
    )
    .await;
    let variables: Option<String> = None;
    let response = GraphQLRequest::new(&bridge, (query.as_str(), variables))?
        .send()
        .await?;
    let parsed_response = response.parse_graphql_response::<GqlResponse>()?;

    assert!(parsed_response.is_err());
    assert!(parsed_response.has_parsed_data());
    assert_eq!(1, parsed_response.get_errors().len());

    Ok(())
}

#[tokio::test]
async fn error_response_parser_with_error() -> Result<(), Box<dyn Error>> {
    let query = file_content("graphql/hero.graphql");
    let (_m, bridge) = create_gql_bridge(200, query.as_str(), file_content("graphql/error.json").as_str()).await;
    let variables: Option<String> = None;
    let response = GraphQLRequest::new(&bridge, (query.as_str(), variables))?
        .send()
        .await?;
    let parsed_response = response.parse_graphql_response::<GqlResponse>()?;

    assert!(parsed_response.is_err());
    assert!(!parsed_response.has_parsed_data());
    assert_eq!(1, parsed_response.get_errors().len());

    Ok(())
}

async fn create_gql_bridge(status_code: usize, query: &str, body: &str) -> (Auth0Mocks, Bridge) {
    let mut mocks = Auth0Mocks::new();
    let url = Url::parse(mockito::server_url().as_str()).unwrap();
    let bridge: Bridge = Bridge::builder().with_auth0(config()).await.build(url);

    let graphql_mock: Mock = mock("POST", "/")
        .match_header("content-type", "application/json")
        .match_header(
            reqwest::header::AUTHORIZATION.as_str(),
            bridge.token().unwrap().to_bearer().as_str(),
        )
        .match_body(Matcher::Json(json!({ "query": query })))
        .with_status(status_code)
        .with_body(body)
        .create();

    mocks.set_endpoint_mock(graphql_mock);

    (mocks, bridge)
}

fn file_content(path_relative_to_recourses: &str) -> String {
    let mut base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_dir.push("tests/resources");
    std::fs::read_to_string(format!("{}/{}", base_dir.to_str().unwrap(), path_relative_to_recourses)).unwrap()
}
