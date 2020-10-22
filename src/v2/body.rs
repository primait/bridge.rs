#[derive(Clone, Debug)]
pub struct Body {
    inner: Vec<u8>,
}

impl Default for Body {
    fn default() -> Self {
        Self { inner: vec![] }
    }
}

impl<T: ToString> From<T> for Body {
    fn from(val: T) -> Self {
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
