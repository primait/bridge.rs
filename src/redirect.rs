use reqwest::redirect::Policy as ReqwestPolicy;

/// Determines how to handle HTTP redirects (3xx responses).
pub enum RedirectPolicy {
    /// Don't follow redirects. Return an error in case of a redirect response.
    NoFollow,
    /// Follow a limited number of redirects. Return an error if the maximum number of redirects is reached.
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
