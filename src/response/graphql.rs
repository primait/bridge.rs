use crate::errors::PrimaBridgeError;
use crate::Response;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Deserialize)]
pub struct GraphQlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<Error>>,
}

impl<T> TryFrom<&Response> for GraphQlResponse<T>
where
    for<'de> T: Deserialize<'de>,
{
    type Error = PrimaBridgeError;

    fn try_from(response: &Response) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(std::str::from_utf8(
            response.raw_body(),
        )?)?)
    }
}

#[derive(Deserialize)]
pub struct Error {
    message: String,
    locations: Option<Vec<Location>>,
    path: Option<Vec<PathSegment>>,
    extensions: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct Location {
    line: u32,
    column: u32,
}

#[derive(Deserialize)]
pub enum PathSegment {
    String(String),
    Num(u32),
}
