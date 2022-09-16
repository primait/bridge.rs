use std::borrow::Cow;

use reqwest::multipart::Part;
use serde::Serialize;

use crate::prelude::{PrimaBridgeError, PrimaBridgeResult};

#[derive(Debug)]
/// A request body.
/// 
/// It can be an in-memory buffer, a file, or a stream.
/// 
/// See the various `From` implementations for ways to construct a `Body`.
pub struct Body {
    pub(crate) inner: reqwest::Body,
}

impl Body {
    /// Construct a `Body` from a streamable type.
    ///
    /// Generally, you can just use the various `From` implementations to create a `Body` instead.
    ///
    /// This is provided for cases where you don't want to load the entire body into memory at once,
    /// and instead want to stream it from something like a file handle.
    pub fn from_stream<S>(stream: S) -> Body
    where
        S: futures::stream::TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        Self {
            inner: reqwest::Body::wrap_stream(stream),
        }
    }

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

impl From<&'static str> for Body {
    fn from(val: &'static str) -> Self {
        Self {
            inner: val.to_string().into_bytes().into(),
        }
    }
}

impl From<&'static [u8]> for Body {
    fn from(val: &'static [u8]) -> Self {
        Self {
            inner: val.to_vec().into(),
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Self { inner: value.into() }
    }
}

impl From<tokio::fs::File> for Body {
    fn from(file: tokio::fs::File) -> Self {
        Self { inner: file.into() }
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
/// A multipart-form file, containing the file's desired name, its MIME type, and its contents as an in-memory buffer or stream.
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
/// A named multipart-form field which contains a file.
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
