# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Next]

## [0.12.0]

### Removed

- `blocking` feature. The library is now async only. (*breaking change*)
- the old `new` and `with_user_agent` functions deprecated in 0.7.3

### Added

- graphql multipart request as specified [here](https://github.com/jaydenseric/graphql-multipart-request-spec).
- examples for rest, graphql, graphql multipart and generic request with auth0 authentication.

## [0.11.0]

- opentelemetry updated to version 0.17. **Careful!!!** The opentelemetry version in your project should match the one in this library (*breaking change*)
- various dependency updates
- removed log dependency, use tracing everywhere

## [0.10.0]

- dashmap dependency updated (<https://github.com/primait/bridge.rs/pull/64>)

## [0.9.2]

- test fix

## [0.9.1]

- lint fix

## [0.9.0]

- Rust 1.56 & Edition 2021
- Update tracing-opentelemetry to 0.16
- Other deps updated
- Documentation now shows required features

## [0.8.0]

- added new function `with_auth0` in bridge builder. This enables jwt authentication to called endpoint.

## [0.7.3]

- deprecated the `new` and `with_user_agent` functions in favor of a builder
- opentelemetry updated to version 0.16. **Careful!!!** The opentelemetry version in your project should match the one in this library
- several dependencies updated
- docker file based on Rust 1.54

## [0.7.2]

### features

- fixes the double header issue (#17)
- adds support for the ```gzip``` feature, which decompress the response content based on the content-type header
- proper handling of graphql errors with a new `ParsedGraphqlResponse` type
- adds ability to specify a request timeout

## [0.7.1]

### breaking change

- opentelemetry updated to version 0.13. **Careful!!!** The opentelemetry version in your project should match the one in this library
- several dependencies updated
- docker file based on Rust 1.51

## [0.7.0]

### breaking change

- opentelemetry updated to version 0.12. **Careful!!!** The opentelemetry version in your project should match the one in this library
- several dependencies updated

### bugfix

- tokio and tokio-test are now dev-dependencies

## [0.6.0]

### breaking change

- bump tokio to 1.0, which brings rustc minimum version to 1.45

## [0.5.1]

### features

- relaxed the uuid requirement

## [0.5.0]

### breaking change

- opentelemetry updated to version 0.11. **Careful!!!** The opentelemetry version in your project should match the one in this library
- tracing-opentelemetry updated to version 0.10. **Careful!!!** The tracing-opentelemetry version in your project should match the one in this library

## [0.4.1]

### features

- adds the ability to set user-agent header

## [0.4.0]

### breaking change

- opentelemetry updated to version 0.10.0. **Careful!!!** The opentelemetry version in your project should match the one in this library

## [0.3.0]

### features

- adds support for raw body response ([PR #21](https://github.com/primait/bridge.rs/pull/21))

### breaking change

- removed the bridge.request api (deprecated) in favor of `Request::get` or `RestRequest::new` and `GraphQLRequest::new` functions.

## [0.2.4]

### bugfix

- fixes the test suite. No impact on the api.

## [0.2.3]

### bugfix

- double content-type header for graphql request

## [0.2.2]

### bugfix

- better handling of headers

## [0.2.1]

### features

- adds back the `with_query_pair` and `with_query_pairs` api

## [0.2.0]

### features

- adds the ability to use a binary as the body of a request. The api has changed:

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

[Next]: https://github.com/primait/bridge.rs/compare/0.12.0...HEAD
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
[0.3.1]: https://github.com/primait/bridge.rs/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/primait/bridge.rs/compare/0.2.4...0.3.0
[0.2.4]: https://github.com/primait/bridge.rs/compare/0.2.3...0.2.4
[0.2.3]: https://github.com/primait/bridge.rs/compare/0.2.2...0.2.3
[0.2.2]: https://github.com/primait/bridge.rs/compare/0.2.1...0.2.2
[0.2.1]: https://github.com/primait/bridge.rs/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/primait/bridge.rs/releases/tag/0.2.0
