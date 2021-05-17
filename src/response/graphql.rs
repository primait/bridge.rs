/// graphql response types and parsers
use crate::errors::PrimaBridgeError;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;

/// A type returned from [get_graphql_response](struct.Response.html#method.get_graphql_response) function that could be either
#[derive(Debug)]
pub enum ParsedGraphqlResponse<T> {
    /// `T` is the deserialized data
    Ok(T),
    Err(PossiblyParsedData<T>),
}

impl<T> ParsedGraphqlResponse<T> {
    pub fn is_ok(&self) -> bool {
        matches!(self, ParsedGraphqlResponse::Ok(_))
    }

    pub fn get_errors(&self) -> Vec<Error> {
        match self {
            ParsedGraphqlResponse::Ok(_) => vec![],
            ParsedGraphqlResponse::Err(possibly_parsed_data) => match possibly_parsed_data {
                PossiblyParsedData::ParsedData(_, errors) => errors.to_vec(),
                PossiblyParsedData::UnparsedData(_, errors) => errors.to_vec(),
                PossiblyParsedData::EmptyData(errors) => errors.to_vec(),
            },
        }
    }

    pub fn has_parsed_data(&self) -> bool {
        matches!(
            self,
            ParsedGraphqlResponse::Ok(_)
                | ParsedGraphqlResponse::Err(PossiblyParsedData::ParsedData(..))
        )
    }
}

#[derive(Deserialize, Debug)]
struct GraphQlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<Error>>,
}

impl<T> From<GraphQlResponse<T>> for ParsedGraphqlResponse<T> {
    fn from(gql_response: GraphQlResponse<T>) -> Self {
        match (gql_response.data, gql_response.errors) {
            (Some(t), None) => Self::Ok(t),
            (Some(t), Some(errors)) => Self::Err(PossiblyParsedData::ParsedData(t, errors)),
            (None, Some(errors)) => Self::Err(PossiblyParsedData::EmptyData(errors)),
            // this should not happen!
            _ => Self::Err(PossiblyParsedData::EmptyData(vec![])),
        }
    }
}

impl<T> TryFrom<&str> for ParsedGraphqlResponse<T>
where
    for<'de> T: Deserialize<'de>,
{
    type Error = PrimaBridgeError;

    fn try_from(body_as_str: &str) -> Result<Self, Self::Error> {
        let result: serde_json::Result<GraphQlResponse<T>> = serde_json::from_str(body_as_str);
        match result {
            Ok(t) => Ok(t.into()),
            Err(e) => {
                dbg!(e);
                let value: Value = serde_json::from_str(body_as_str)?;
                Ok(ParsedGraphqlResponse::Err(
                    PossiblyParsedData::UnparsedData(value, vec![]),
                ))
            }
        }
    }
}

/// a struct representing a graphql error
/// see: https://spec.graphql.org/June2018/#sec-Errors
#[derive(Deserialize, Debug, Clone)]
pub struct Error {
    message: String,
    locations: Option<Vec<Location>>,
    path: Option<Vec<PathSegment>>,
    extensions: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Location {
    line: u32,
    column: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PathSegment {
    String(String),
    Num(u32),
}

/// an error type to represent all possible outcome for a graphql response deserialization
#[derive(Debug)]
pub enum PossiblyParsedData<T> {
    /// a tuple type with the converted `T` type and a vector of [Error]. This could happens thanks to the fact that graphql can return null for a nullable type, even if there was an error while resolving the data.
    ParsedData(T, Vec<Error>),
    /// a tuple with the [serde_json::Value] of the Response, and a vector of [Error]
    UnparsedData(Value, Vec<Error>),
    /// a vector of [Error]. This means that the response contained just the "error" part, without the data.
    EmptyData(Vec<Error>),
}
