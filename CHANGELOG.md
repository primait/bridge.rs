# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security

- Switched to using XChaCha20Poly1305 for the redis token cache encryption.

This addresses many a few medium severity security issues with the tokens

---

## [0.16.4] - 2024-07-04

### Added

- Support for opentelemetry 0.23, now the default version.
- `tracing_opentelemetry_0_23` feature

---

## [0.16.3] - 2024-05-22

### Fixed

- The authority server successful response might not include the `scope` field,
  that is now optional.

---

## [0.16.2] - 2024-05-10

### Added

- Support for opentelemetry 0.22
- `tracing_opentelemetry_0_22` feature
- `tracing_opentelemetry` is now an alias for the latest version of otel(so
  `tracing_opentelemetry_0_22`)

Opentelemetry 0.20 or 0.21 support can be enabled by enabling the
`tracing_opentelemetry_0_20` or `tracing_opentelemetry_0_21` features
respectively instead of tracing_opentelemetry.

---

## [0.16.1] - 2024-05-02

### Changed

- Added jwks_client_rs instead of reimplementing it's functionality
- Updated reqwest to 0.12, reqwest-middleware to 0.3 and http to 1.0

---

## [0.16.0] - 2024-03-11

### Added

- Support for opentelemetry 0.21
- `tracing_opentelemetry_0_20` and `tracing_opentelemetry_0_21` features
- `tracing_opentelemetry` is now an alias for the latest version of otel(so
  `tracing_opentelemetry_0_21`)

Opentelemetry 0.20 support can be enabled by enabling the
`tracing_opentelemetry_0_20` feature instead of tracing_opentelemetry. We are
going to support at least the last 3 versions of opentelemetry. After that we
mightremove support for older otel version without it being a breaking change.

---

## [0.15.1] - 2023-10-20

### Added

- `scope` for auth0 token request

---

## [0.15.0] - 2023-09-20

### Added

- Support for reqwest_middleware

### Changed

- MSRV is 1.72, for https://github.com/rust-lang/rust/issues/107557

---

## [0.14.6] - 2023-08-24

### Fixed

- `otel` module is imported only if `tracing_opentelemetry` feature is defined

---

## [0.14.5] - 2023-08-23

### Changed

- Allow opentelemetry 0.20

---

## [0.14.4] - 2023-05-29

### Changed

- Updates Auth0 token refresh logic to reduce the number of token renewals
  (#122)

---

## [0.14.3] - 2022-11-02

### Added

- Added method to customize `pool_max_idle_per_host` from the builder

## [0.14.2] - 2022-10-04

### Added

- Introduced the new `redis-tls` feature that enables the tls authentication for
  the redis client

## [0.14.1] - 2022-09-28

### Added

- Reintroduced `{RestRequest, GraphQLRequest}.get_body` for public use.\
  The method was publicly accessible before 0.14.0 but was undocumented and
  meant only for internal use.
- Added `BridgeBuilder.with_redirect_policy` to specify how the Bridge should
  handle HTTP redirects

---

## [0.14.0] - 2022-09-26

### Added

- Added the ability to send files via REST multipart requests by using the new
  `RestRequest.multipart_body` method.\
  Refer to the example in the repository to see how to use the new APIs.
- Added `{RestRequest, GraphQLRequest}.with_custom_header` to set a single
  custom header for a request

### Changed

- **(BREAKING)** `Multipart` has been renamed into `GraphQLMultipart` to avoid
  ambiguity with `RestMultipart`
- `MultipartFile::new` now accepts anything that can be converted into a `Body`
  instead of just a `Vec<u8>`.\
  This allows to provide the file content from a stream, which can be useful in
  case you want to send a file, but you don't want to load it in memory all at
  once.
  - To better support this use case, a
    [tokio::fs::File](https://docs.rs/tokio/latest/tokio/fs/struct.File.html)
    can now be converted into a `Body` too
- Reduced cloning and allocations throughout the library
- Re-exported more types at the top level of the crate

### Removed

- **(BREAKING)** `{RestRequest, GraphQLRequest}.add_custom_headers` and
  `set_custom_headers`: use `with_custom_headers` instead
- `(GraphQL)Multipart.into_form`: it is a private API that wasn't meant to be
  exported publicly
- `async` feature: it was enabled by default and disabling it caused compilation
  to break

---

## [0.13.1] - 2022-06-23

### Added

- New function `status_code` to get `StatusCode` from `Response`.

---

## [0.13.0]

### Changed

- removed `block_modes` deprecated dependency in favour of the new `cbc`
  dependency
- broaden the dependency on uuid to support 1.x versions

---

## [0.12.0]

### Removed

- `blocking` feature. The library is now async only. (_breaking change_)
- the old `new` and `with_user_agent` functions deprecated in 0.7.3

### Added

- graphql multipart request as specified
  [here](https://github.com/jaydenseric/graphql-multipart-request-spec).
- examples for rest, graphql, graphql multipart and generic request with auth0
  authentication.

---

## [0.11.0]

- opentelemetry updated to version 0.17. **Careful!!!** The opentelemetry
  version in your project should match the one in this library (_breaking
  change_)
- various dependency updates
- removed log dependency, use tracing everywhere

---

## [0.10.0]

- dashmap dependency updated (<https://github.com/primait/bridge.rs/pull/64>)

---

## [0.9.2]

- test fix

---

## [0.9.1]

- lint fix

---

## [0.9.0]

- Rust 1.56 & Edition 2021
- Update tracing-opentelemetry to 0.16
- Other deps updated
- Documentation now shows required features

---

## [0.8.0]

- added new function `with_auth0` in bridge builder. This enables jwt
  authentication to called endpoint.

---

## [0.7.3]

- deprecated the `new` and `with_user_agent` functions in favor of a builder
- opentelemetry updated to version 0.16. **Careful!!!** The opentelemetry
  version in your project should match the one in this library
- several dependencies updated
- docker file based on Rust 1.54

---

## [0.7.2]

### features

- fixes the double header issue (#17)
- adds support for the `gzip` feature, which decompress the response content
  based on the content-type header
- proper handling of graphql errors with a new `ParsedGraphqlResponse` type
- adds ability to specify a request timeout

---

## [0.7.1]

### breaking change

- opentelemetry updated to version 0.13. **Careful!!!** The opentelemetry
  version in your project should match the one in this library
- several dependencies updated
- docker file based on Rust 1.51

---

## [0.7.0]

### breaking change

- opentelemetry updated to version 0.12. **Careful!!!** The opentelemetry
  version in your project should match the one in this library
- several dependencies updated

### bugfix

- tokio and tokio-test are now dev-dependencies

---

## [0.6.0]

### breaking change

- bump tokio to 1.0, which brings rustc minimum version to 1.45

---

## [0.5.1]

### features

- relaxed the uuid requirement

---

## [0.5.0]

### breaking change

- opentelemetry updated to version 0.11. **Careful!!!** The opentelemetry
  version in your project should match the one in this library
- tracing-opentelemetry updated to version 0.10. **Careful!!!** The
  tracing-opentelemetry version in your project should match the one in this
  library

---

## [0.4.1]

### features

- adds the ability to set user-agent header

---

## [0.4.0]

### breaking change

- opentelemetry updated to version 0.10.0. **Careful!!!** The opentelemetry
  version in your project should match the one in this library

---

## [0.3.0]

### features

- adds support for raw body response
  ([PR #21](https://github.com/primait/bridge.rs/pull/21))

### breaking change

- removed the bridge.request api (deprecated) in favor of `Request::get` or
  `RestRequest::new` and `GraphQLRequest::new` functions.

---

## [0.2.4]

### bugfix

- fixes the test suite. No impact on the api.

---

## [0.2.3]

### bugfix

- double content-type header for graphql request

---

## [0.2.2]

### bugfix

- better handling of headers

---

## [0.2.1]

### features

- adds back the `with_query_pair` and `with_query_pairs` api

---

## [0.2.0]

### features

- adds the ability to use a binary as the body of a request. The api has
  changed:

**before**

```rust
let body: Option<String> = None;
bridge.request(RequestType::rest(body, Method::GET)).send();
```

**now**

```rust
RestRequest::new(&bridge).send();
// OR
Request::rest(&bridge).send()
```

The old API is still available but deprecated. It will be removed soon.

[Unreleased]: https://github.com/primait/bridge.rs/compare/0.16.3...HEAD
[0.16.3]: https://github.com/primait/bridge.rs/compare/0.16.2...0.16.2
[0.16.2]: https://github.com/primait/bridge.rs/compare/0.16.1...0.16.2
[0.16.1]: https://github.com/primait/bridge.rs/compare/0.16.0...0.16.1
[0.16.0]: https://github.com/primait/bridge.rs/compare/0.15.1-rc.0...0.16.0
[0.15.1]: https://github.com/primait/bridge.rs/compare/0.15.0...0.15.1
[0.15.0]: https://github.com/primait/bridge.rs/compare/0.14.6...0.15.0
[0.14.6]: https://github.com/primait/bridge.rs/compare/0.14.5...0.14.6
[0.14.5]: https://github.com/primait/bridge.rs/compare/0.14.4...0.14.5
[0.14.4]: https://github.com/primait/bridge.rs/compare/0.14.3...0.14.4
[0.14.3]: https://github.com/primait/bridge.rs/compare/0.14.2...0.14.3
[0.14.2]: https://github.com/primait/bridge.rs/compare/0.14.1...0.14.2
[0.14.1]: https://github.com/primait/bridge.rs/compare/0.14.0...0.14.1
[0.14.0]: https://github.com/primait/bridge.rs/compare/0.13.1...0.14.0
[0.13.1]: https://github.com/primait/bridge.rs/compare/0.13.0...0.13.1
[0.13.0]: https://github.com/primait/bridge.rs/compare/0.12.0...0.13.0
[0.12.0]: https://github.com/primait/bridge.rs/compare/0.11.0...0.12.0
[0.11.0]: https://github.com/primait/bridge.rs/compare/0.10.0...0.11.0
[0.10.0]: https://github.com/primait/bridge.rs/compare/0.9.2...0.10.0
[0.9.2]: https://github.com/primait/bridge.rs/compare/0.9.1...0.9.2
[0.9.1]: https://github.com/primait/bridge.rs/compare/0.9.0...0.9.1
[0.9.0]: https://github.com/primait/bridge.rs/compare/0.8.0...0.9.0
[0.8.0]: https://github.com/primait/bridge.rs/compare/0.7.3...0.8.0
[0.7.3]: https://github.com/primait/bridge.rs/compare/0.7.2...0.7.3
[0.7.2]: https://github.com/primait/bridge.rs/compare/0.7.1...0.7.2
[0.7.1]: https://github.com/primait/bridge.rs/compare/0.7.0...0.7.1
[0.7.0]: https://github.com/primait/bridge.rs/compare/0.6.0...0.7.0
[0.6.0]: https://github.com/primait/bridge.rs/compare/0.5.1...0.6.0
[0.5.1]: https://github.com/primait/bridge.rs/compare/0.5.0...0.5.1
[0.5.0]: https://github.com/primait/bridge.rs/compare/0.4.1...0.5.0
[0.4.1]: https://github.com/primait/bridge.rs/compare/0.4.0...0.4.1
[0.4.0]: https://github.com/primait/bridge.rs/compare/0.3.1...0.4.0
[0.3.0]: https://github.com/primait/bridge.rs/compare/0.2.4...0.3.0
[0.2.4]: https://github.com/primait/bridge.rs/compare/0.2.3...0.2.4
[0.2.3]: https://github.com/primait/bridge.rs/compare/0.2.2...0.2.3
[0.2.2]: https://github.com/primait/bridge.rs/compare/0.2.1...0.2.2
[0.2.1]: https://github.com/primait/bridge.rs/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/primait/bridge.rs/releases/tag/0.2.0
