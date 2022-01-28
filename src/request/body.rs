use serde::Serialize;

#[derive(Clone, Debug)]
pub struct Body {
    inner: Vec<u8>,
}

impl From<String> for Body {
    fn from(val: String) -> Self {
        Self {
            inner: val.into_bytes(),
        }
    }
}

impl From<&str> for Body {
    fn from(val: &str) -> Self {
        Self {
            inner: val.to_string().into_bytes(),
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Self { inner: value }
    }
}

impl From<Body> for Vec<u8> {
    fn from(body: Body) -> Self {
        body.inner
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Serialize)]
pub struct GraphQLBody<T> {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<T>,
}

impl<T: Serialize> From<(&str, Option<T>)> for GraphQLBody<T> {
    fn from((query, variables): (&str, Option<T>)) -> Self {
        Self {
            query: query.to_owned(),
            variables,
        }
    }
}

impl<T: Serialize> From<(String, Option<T>)> for GraphQLBody<T> {
    fn from((query, variables): (String, Option<T>)) -> Self {
        (query.as_str(), variables).into()
    }
}

impl<T: Serialize> From<(String, T)> for GraphQLBody<T> {
    fn from((query, variables): (String, T)) -> Self {
        (query.as_str(), Some(variables)).into()
    }
}
