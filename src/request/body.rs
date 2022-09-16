use std::borrow::Cow;

use reqwest::multipart::Part;
use serde::Serialize;

use crate::prelude::{PrimaBridgeError, PrimaBridgeResult};

#[derive(Debug)]
pub struct Body {
    pub(crate) inner: reqwest::Body,
}

impl Body {
    #[cfg(test)]
    pub(crate) fn as_str(&self) -> Option<Cow<'_, str>> {
        self.inner.as_bytes().map(String::from_utf8_lossy)
    }
}

impl Default for Body {
    fn default() -> Self {
        Self {
            inner: reqwest::Body::from(Vec::new()),
        }
    }
}

impl From<String> for Body {
    fn from(val: String) -> Self {
        Self {
            inner: val.into_bytes().into(),
        }
    }
}

impl From<&str> for Body {
    fn from(val: &str) -> Self {
        Self {
            inner: val.to_string().into_bytes().into(),
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Self { inner: value.into() }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct GraphQLBody<T> {
    pub(crate) query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) variables: Option<T>,
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

#[derive(Debug)]
pub struct MultipartFile {
    pub(crate) body: Body,
    pub(crate) name_opt: Option<String>,
    pub(crate) mime_type_opt: Option<String>,
}

impl MultipartFile {
    pub fn new(body: impl Into<Body>) -> Self {
        Self {
            body: body.into(),
            name_opt: None,
            mime_type_opt: None,
        }
    }

    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name_opt: Some(name.into()),
            ..self
        }
    }

    pub fn with_mime_type(self, mime_type: impl Into<String>) -> Self {
        Self {
            mime_type_opt: Some(mime_type.into()),
            ..self
        }
    }

    pub(crate) fn into_part(self) -> PrimaBridgeResult<Part> {
        let mut part = Part::stream(self.body.inner);
        if let Some(name) = self.name_opt {
            part = part.file_name(name);
        }
        if let Some(mime) = self.mime_type_opt {
            part = part
                .mime_str(mime.as_str())
                .map_err(|_| PrimaBridgeError::InvalidMultipartFileMimeType(mime.to_string()))?;
        }
        Ok(part)
    }
}

#[derive(Debug)]
pub struct MultipartFormFileField {
    pub(crate) field_name: Cow<'static, str>,
    pub(crate) file: MultipartFile,
}
impl MultipartFormFileField {
    pub fn new<S>(file_name: S, file: MultipartFile) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            field_name: file_name.into(),
            file,
        }
    }
}
impl std::hash::Hash for MultipartFormFileField {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.field_name.hash(state);
    }
}
impl Eq for MultipartFormFileField {}
impl PartialEq for MultipartFormFileField {
    fn eq(&self, other: &Self) -> bool {
        self.field_name == other.field_name
    }
}
