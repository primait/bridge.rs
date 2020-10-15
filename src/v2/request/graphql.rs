use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::v2::Body;
use crate::Bridge;

pub struct GraphQLRequest<'a> {
    bridge: &'a Bridge,
    body: Option<Body>,
}

impl<'a> GraphQLRequest<'a> {
    pub fn new(bridge: &'a Bridge) -> Self {
        Self {
            bridge,
            body: Default::default(),
        }
    }
}
