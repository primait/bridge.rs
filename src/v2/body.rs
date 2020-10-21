use crate::errors::PrimaBridgeError;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct Body {
    inner: Vec<u8>,
}

impl Default for Body {
    fn default() -> Self {
        Self { inner: vec![] }
    }
}

impl TryFrom<&str> for Body {
    type Error = PrimaBridgeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: value.to_owned().into_bytes(),
        })
    }
}

impl TryFrom<String> for Body {
    type Error = PrimaBridgeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: value.into_bytes(),
        })
    }
}

impl TryFrom<Option<String>> for Body {
    type Error = PrimaBridgeError;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Body::default()),
            Some(val) => Ok(Self {
                inner: val.into_bytes(),
            }),
        }
    }
}

impl TryFrom<Vec<u8>> for Body {
    type Error = PrimaBridgeError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self { inner: value })
    }
}

impl From<Body> for Vec<u8> {
    fn from(body: Body) -> Self {
        body.inner
    }
}
