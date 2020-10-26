[![Build Status](https://drone-1.prima.it/api/badges/primait/bridge.rs/status.svg)](https://drone-1.prima.it/primait/bridge.rs)

# bridge.rs
Prima bridge pattern implementation for rust

### Example


```rust
use prima_bridge::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct DeserializableData {
    test: String
}

// using we make sure that `Bridge` get instantiated only once
fn bridge() -> &'static Bridge {
    static BRIDGE: OnceCell<Bridge> = OnceCell::new();
    BRIDGE.get_or_init(|| Bridge::new("https://prima.it/api"))
}

pub fn fetch_data() -> YourResult<DeserializableData> {
    Request::get(bridge())
        .send()?
        .get_data(&["nested", "selector"])? // response is {"nested": {"selector": {"data": "test"}}}
}           
```

To understand this example you should know:
 - [once_cell](https://crates.io/crates/once_cell) library providing the cell type
 - Rust error handling to use ? and convert it to a custom error type. See for example [thiserror](https://crates.io/crates/thiserror)