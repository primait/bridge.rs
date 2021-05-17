use crate::errors::PrimaBridgeError;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;

pub enum ParsedGraphqlResponse<T> {
    Ok(T),
    Err(PossiblyParsedData<T>),
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
            Err(_) => {
                let value: Value = serde_json::from_str(body_as_str)?;
                Ok(ParsedGraphqlResponse::Err(
                    PossiblyParsedData::UnparsedData(value, vec![]),
                ))
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Error {
    message: String,
    locations: Option<Vec<Location>>,
    path: Option<Vec<PathSegment>>,
    extensions: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct Location {
    line: u32,
    column: u32,
}

#[derive(Deserialize, Debug)]
pub enum PathSegment {
    String(String),
    Num(u32),
}

pub enum PossiblyParsedData<T> {
    ParsedData(T, Vec<Error>),
    UnparsedData(Value, Vec<Error>),
    EmptyData(Vec<Error>),
}
