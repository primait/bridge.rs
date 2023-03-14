use std::{collections::HashMap, fmt::Debug};

use serde::Deserialize;
use serde_json::Value;

/// graphql response types and parsers
use crate::errors::PrimaBridgeError;

/// A type returned from [parse_graphql_response](crate::Response#parse_graphql_response) function useful for getting
/// full control of a GraphQL response.
pub type ParsedGraphqlResponse<T> = Result<T, PossiblyParsedData<T>>;

pub trait ParsedGraphqlResponseExt<T>
where
    for<'de> T: Deserialize<'de>,
{
    fn from_str(body_as_str: &str) -> Result<Self, PrimaBridgeError>
    where
        Self: Sized;

    fn get_errors(&self) -> Vec<Error>;
    fn has_parsed_data(&self) -> bool;
}

impl<T> ParsedGraphqlResponseExt<T> for ParsedGraphqlResponse<T>
where
    for<'de> T: Deserialize<'de>,
{
    fn from_str(body_as_str: &str) -> Result<Self, PrimaBridgeError> {
        let result: serde_json::Result<GraphqlResponse<T>> = serde_json::from_str(body_as_str);
        match result {
            Ok(t) => Ok(t.into()),
            Err(_e) => {
                let value: Value = serde_json::from_str(body_as_str)?;
                Ok(Err(PossiblyParsedData::UnparsedData(value, vec![])))
            }
        }
    }

    fn get_errors(&self) -> Vec<Error> {
        match self {
            Ok(_) => vec![],
            Err(possibly_parsed_data) => match possibly_parsed_data {
                PossiblyParsedData::ParsedData(_, errors) => errors.to_vec(),
                PossiblyParsedData::UnparsedData(_, errors) => errors.to_vec(),
                PossiblyParsedData::EmptyData(errors) => errors.to_vec(),
            },
        }
    }

    fn has_parsed_data(&self) -> bool {
        matches!(
            self,
            ParsedGraphqlResponse::Ok(_) | ParsedGraphqlResponse::Err(PossiblyParsedData::ParsedData(..))
        )
    }
}

#[derive(Deserialize, Debug)]
struct GraphqlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<Error>>,
}

impl<T> From<GraphqlResponse<T>> for ParsedGraphqlResponse<T> {
    fn from(gql_response: GraphqlResponse<T>) -> Self {
        match (gql_response.data, gql_response.errors) {
            (Some(t), None) => Ok(t),
            (Some(t), Some(errors)) => Err(PossiblyParsedData::ParsedData(t, errors)),
            (None, Some(errors)) => Err(PossiblyParsedData::EmptyData(errors)),
            // this should not happen!
            _ => Err(PossiblyParsedData::EmptyData(vec![])),
        }
    }
}

/// A struct representing a GraphQL error according to the [GraphQL specification](https://spec.graphql.org/June2018/#sec-Errors).
#[derive(Deserialize, Debug, Clone)]
pub struct Error {
    pub message: String,
    pub locations: Option<Vec<Location>>,
    pub path: Option<Vec<PathSegment>>,
    pub extensions: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Location {
    pub line: u32,
    pub column: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PathSegment {
    String(String),
    Num(u32),
}

/// An error type to represent all possible outcomes of a GraphQL response deserialization.
#[derive(Debug)]
pub enum PossiblyParsedData<T> {
    /// a tuple type with the converted `T` type and a vector of [Error]. This could happen due to the fact that
    /// GraphQL can return null for a nullable type, even if there was an error while resolving the data.
    ParsedData(T, Vec<Error>),
    /// a tuple with the [serde_json::Value] of the Response, and a vector of [Error]
    UnparsedData(Value, Vec<Error>),
    /// a vector of [Error]. This means that the response contained just the "error" part, without the data.
    EmptyData(Vec<Error>),
}
