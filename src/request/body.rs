use std::borrow::Cow;

use reqwest::multipart::Part;
use serde::Serialize;

use crate::prelude::{PrimaBridgeError, PrimaBridgeResult};

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

impl From<&Body> for String {
    fn from(body: &Body) -> Self {
        String::from_utf8_lossy(&body.inner).to_string()
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

#[derive(Debug, Clone)]
pub struct MultipartFile {
    pub(crate) bytes: Vec<u8>,
    pub(crate) name_opt: Option<String>,
    pub(crate) mime_type_opt: Option<String>,
}

impl MultipartFile {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
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
        let len = self.bytes.len() as u64;
        let mut part = Part::stream_with_length(self.bytes, len);
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
