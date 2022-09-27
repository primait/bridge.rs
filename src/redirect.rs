use reqwest::redirect::Policy as ReqwestPolicy;
pub enum Policy {
    NoFollow,
    Limited(usize),
}

impl From<Policy> for ReqwestPolicy {
    fn from(policy: Policy) -> Self {
        match policy {
            Policy::NoFollow => ReqwestPolicy::none(),
            Policy::Limited(max) => ReqwestPolicy::limited(max),
        }
    }
}
