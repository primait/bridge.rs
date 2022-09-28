use reqwest::redirect::Policy as ReqwestPolicy;
pub enum RedirectPolicy {
    NoFollow,
    Limited(usize),
}

impl From<RedirectPolicy> for ReqwestPolicy {
    fn from(policy: RedirectPolicy) -> Self {
        match policy {
            RedirectPolicy::NoFollow => ReqwestPolicy::none(),
            RedirectPolicy::Limited(max) => ReqwestPolicy::limited(max),
        }
    }
}
