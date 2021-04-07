# Changelog

### 0.7.1
#####breaking change
- opentelemetry updated to version 0.13. **Careful!!!** The opentelemetry version in your project should match the one in this library
- several dependencies updated
- docker file based on Rust 1.51

### 0.7.0
#####breaking change
- opentelemetry updated to version 0.12. **Careful!!!** The opentelemetry version in your project should match the one in this library
- several dependencies updated    
#####bugfix
- tokio and tokio-test are now dev-dependencies

### 0.6.0
#####breaking change
- bump tokio to 1.0, which brings rustc minimum version to 1.45

### 0.5.1
#####features
- relaxed the uuid requirement

### 0.5.0
#####breaking change
- opentelemetry updated to version 0.11. **Careful!!!** The opentelemetry version in your project should match the one in this library
- tracing-opentelemetry updated to version 0.10. **Careful!!!** The tracing-opentelemetry version in your project should match the one in this library

### 0.4.1
#####features
- adds the ability to set user-agent header

### 0.4.0
#####breaking change
- opentelemetry updated to version 0.10.0. **Careful!!!** The opentelemetry version in your project should match the one in this library

### 0.3.0
#####features
- adds support for raw body response ([PR #21](https://github.com/primait/bridge.rs/pull/21))

#####breaking change
- removed the bridge.request api (deprecated) in favor of `Request::get` or `RestRequest::new` and `GraphQLRequest::new` functions.

### 0.2.4
#####bugfix
- fixes the test suite. No impact on the api.

### 0.2.3
#####bugfix
- double content-type header for graphql request

### 0.2.2
#####bugfix
- better handling of headers

### 0.2.1
#####features
- adds back the `with_query_pair` and `with_query_pairs` api

### 0.2.0
#####features
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
