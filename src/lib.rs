#![cfg_attr(docsrs, feature(doc_cfg))]

//! This crate gives a high level API to execute external HTTP requests.
//!
//! It is supposed to give the basics building blocks for building bridges to other services
//! while abstracting the low level stuff like adding custom headers and request tracing.
//!
//! It supports both REST and GraphQL requests.
//!
//! You should start by creating a [Bridge] instance.
//! This instance should live for all the application lifetime.
//!
//! **Do not create a new bridge on every request!**
//!
//! You should use something like [once_cell](https://crates.io/crates/once_cell) or [lazy_static](https://crates.io/crates/lazy_static), or some sort of inversion of control container to
//! pass around.
//!
//! The bridge implement a type state pattern to build the external request.
//!
//! ### Features
//! * `auth0` - enable auth0 integration, allowing bridge.rs to retrieve tokens from auth0  for authentication
//! * `gzip` - provides response body gzip decompression.
//! * `redis-tls` - add support for connecting to redis with tls
//! * `grpc` - provides the [GrpcOtelInterceptor] for adding the opentelemetry context to the gRPC requests
//! * `tracing_opentelemetry` - adds support for integration with opentelemetry.
//!   This feature is an alias for the latest `tracing_opentelemetry_x_xx` feature.
//! * `tracing_opentelemetry_x_xx` (e.g. `tracing_opentelemetry_0_27`) - adds support for integration with a particular opentelemetry version.
//!   We are going to support at least the last 3 versions of opentelemetry. After that we might remove support for older otel version without it being a breaking change.

use auth0::RefreshingToken;
use errors::PrimaBridgeError;
use http::{header::HeaderName, HeaderValue, Method};
use reqwest::{multipart::Form, Url};
use sealed::Sealed;

pub use self::{
    builder::BridgeBuilder,
    redirect::RedirectPolicy,
    request::{
        Body, DeliverableRequest, GraphQLMultipart, GraphQLRequest, MultipartFile, MultipartFormFileField, Request,
        RestMultipart, RestRequest,
    },
    response::graphql::{Error, ParsedGraphqlResponse, ParsedGraphqlResponseExt, PossiblyParsedData},
    response::Response,
};
#[cfg(all(feature = "grpc", feature = "_any_otel_version"))]
pub use request::grpc::{GrpcOtelInterceptedService, GrpcOtelInterceptor};

pub mod builder;
mod errors;
pub mod prelude;
mod redirect;
mod request;
mod response;

#[cfg(feature = "auth0")]
#[cfg_attr(docsrs, doc(cfg(feature = "auth0")))]
pub mod auth0;

/// The basic Bridge type, using a [reqwest::Client] as the client.
pub type Bridge = BridgeImpl<reqwest::Client>;

/// A Bridge instance that's generic across the client. If the [BridgeBuilder] is used
/// to construct a bridge with middleware, this type will be used to wrap the [reqwest_middleware::ClientWithMiddleware].
#[derive(Debug, Clone)]
pub struct BridgeImpl<T: BridgeClient> {
    inner_client: T,
    endpoint: Url,
    #[cfg(feature = "auth0")]
    auth0_opt: Option<RefreshingToken>,
}

/// A trait that abstracts the client used by the [BridgeImpl], such that both reqwest clients and reqwest
/// clients with middleware can be used, more or less interchangeably.
#[doc(hidden)]
pub trait BridgeClient: Sealed + Clone {
    type Builder: PrimaRequestBuilderInner;
    fn request(&self, method: Method, url: Url) -> PrimaRequestBuilder<Self::Builder>;
}

/// A trait which abstracts across request builders, to allow for both reqwest and reqwest with middleware
/// request builders to be used.
#[doc(hidden)]
#[async_trait::async_trait]
pub trait PrimaRequestBuilderInner: Send + Sealed {
    fn timeout(self, timeout: std::time::Duration) -> Self;
    fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>;
    fn headers(self, headers: http::HeaderMap) -> Self;

    fn body(self, body: impl Into<reqwest::Body>) -> Self;
    fn multipart(self, multipart: Form) -> Self;
    async fn send(self, url: Url) -> Result<reqwest::Response, PrimaBridgeError>;
}

/// A wrapper around a generic request builder
#[doc(hidden)]
pub struct PrimaRequestBuilder<T: PrimaRequestBuilderInner> {
    url: Url,
    inner: T,
}

impl BridgeClient for reqwest::Client {
    type Builder = reqwest::RequestBuilder;
    fn request(&self, method: Method, url: Url) -> PrimaRequestBuilder<Self::Builder> {
        PrimaRequestBuilder::new(url.clone(), self.request(method, url))
    }
}

impl BridgeClient for reqwest_middleware::ClientWithMiddleware {
    type Builder = reqwest_middleware::RequestBuilder;
    fn request(&self, method: Method, url: Url) -> PrimaRequestBuilder<Self::Builder> {
        PrimaRequestBuilder::new(url.clone(), self.request(method, url))
    }
}

impl<T: PrimaRequestBuilderInner> PrimaRequestBuilder<T> {
    fn new(url: Url, inner: T) -> Self {
        Self { url, inner }
    }

    fn timeout(self, timeout: std::time::Duration) -> Self {
        Self {
            inner: self.inner.timeout(timeout),
            ..self
        }
    }

    fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        Self {
            inner: self.inner.header(key, value),
            ..self
        }
    }

    fn headers(self, headers: http::HeaderMap) -> Self {
        Self {
            inner: self.inner.headers(headers),
            ..self
        }
    }

    fn body(self, body: impl Into<reqwest::Body>) -> Self {
        Self {
            inner: self.inner.body(body),
            ..self
        }
    }

    fn multipart(self, multipart: Form) -> Self {
        Self {
            inner: self.inner.multipart(multipart),
            ..self
        }
    }

    async fn send(self) -> Result<reqwest::Response, PrimaBridgeError> {
        self.inner.send(self.url).await
    }
}

#[async_trait::async_trait]
impl PrimaRequestBuilderInner for reqwest::RequestBuilder {
    fn timeout(self, timeout: std::time::Duration) -> Self {
        self.timeout(timeout)
    }

    fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.header(key, value)
    }
    fn headers(self, headers: http::HeaderMap) -> Self {
        self.headers(headers)
    }

    fn body(self, body: impl Into<reqwest::Body>) -> Self {
        self.body(body)
    }
    fn multipart(self, multipart: Form) -> Self {
        self.multipart(multipart)
    }
    async fn send(self, url: Url) -> Result<reqwest::Response, PrimaBridgeError> {
        self.send().await.map_err(|e| PrimaBridgeError::HttpError {
            source: e,
            url: url.clone(),
        })
    }
}

#[async_trait::async_trait]
impl PrimaRequestBuilderInner for reqwest_middleware::RequestBuilder {
    fn timeout(self, timeout: std::time::Duration) -> Self {
        self.timeout(timeout)
    }

    fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.header(key, value)
    }
    fn headers(self, headers: http::HeaderMap) -> Self {
        self.headers(headers)
    }

    fn body(self, body: impl Into<reqwest::Body>) -> Self {
        self.body(body)
    }

    fn multipart(self, multipart: Form) -> Self {
        self.multipart(multipart)
    }

    async fn send(self, url: Url) -> Result<reqwest::Response, PrimaBridgeError> {
        self.send().await.map_err(|e| match e {
            reqwest_middleware::Error::Reqwest(e) => PrimaBridgeError::HttpError {
                source: e,
                url: url.clone(),
            },
            reqwest_middleware::Error::Middleware(e) => {
                PrimaBridgeError::MiddlewareError(reqwest_middleware::Error::from(e))
            }
        })
    }
}

impl Bridge {
    /// Creates an instance of a [BridgeBuilder].
    pub fn builder() -> BridgeBuilder {
        BridgeBuilder::create()
    }

    #[cfg(feature = "auth0")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth0")))]
    /// Gets the JWT token used by the Bridge, if it has been configured with Auth0 authentication via [BridgeBuilder.with_auth0](BridgeBuilder#with_auth0).
    pub fn token(&self) -> Option<auth0::Token> {
        self.auth0_opt.as_ref().map(|auth0| auth0.token().clone())
    }
}

mod sealed {
    use crate::BridgeClient;

    pub trait Sealed {}

    impl Sealed for reqwest::Client {}
    impl Sealed for reqwest_middleware::ClientWithMiddleware {}
    impl Sealed for reqwest_middleware::RequestBuilder {}
    impl Sealed for reqwest::RequestBuilder {}
    impl<Client: BridgeClient> Sealed for crate::request::RestRequest<'_, Client> {}
    impl<Client: BridgeClient> Sealed for crate::request::GraphQLRequest<'_, Client> {}
}
