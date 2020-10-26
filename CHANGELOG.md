# Changelog

### 0.2.0
features:
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