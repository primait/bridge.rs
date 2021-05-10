[![Build Status](https://drone-1.prima.it/api/badges/primait/bridge.rs/status.svg)](https://drone-1.prima.it/primait/bridge.rs)

# bridge.rs
Prima bridge pattern implementation for rust

[Api documentation](https://docs.rs/prima_bridge)

### Example


```rust
use serde::Deserialize;
use prima_bridge::prelude::*;
use once_cell::sync::OnceCell;

#[derive(Deserialize, Debug)]
pub struct MyCustomData {
    name: String
}

// using OnceCell we make sure that `Bridge` gets instantiated only once
fn bridge() -> &'static Bridge {
    static BRIDGE: OnceCell<Bridge> = OnceCell::new();
    BRIDGE.get_or_init(|| Bridge::new("https://swapi.dev/api".parse().unwrap()))
}


pub fn fetch_data() -> Result<MyCustomData, PrimaBridgeError> {
    Request::get(bridge())
        .to("people/1")
        .send()?
        .get_data(&[])
}

fn main() {
    let data = fetch_data().expect("there was an error while fetching data");
    println!("the name is {}", data.name);
}       
```

To understand this example you should know:
 - [once_cell](https://crates.io/crates/once_cell) library providing the cell type
 - Rust error handling to use ? and convert it to a custom error type. See for example [thiserror](https://crates.io/crates/thiserror)

### Local development

- **cargo make test** *execute the full test suite*
- **cargo make lint** *execute the full test suite*