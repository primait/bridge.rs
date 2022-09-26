//! Rest example
//!
//! In this example there's the explanation on how to achieve a rest call to an endpoint and parsing
//! the response. The response is the person fetched from persons api with id=1.
//!
//! Run this to run the example:
//!
//! ```shell
//! cargo run --example rest_multipart
//! ```
use std::{collections::HashSet, iter::FromIterator};

use prima_bridge::{prelude::*, MultipartFile, MultipartFormFileField, RestMultipart};

const URL: &str = "https://youtube.com/";

#[tokio::main]
async fn main() {
    let bridge: Bridge = Bridge::builder().build(URL.parse().unwrap());

    let multipart = RestMultipart::multiple(HashSet::from_iter([
        MultipartFormFileField::new(
            "file0",
            MultipartFile::new("Hello, world!")
                .with_name("hello_world.txt")
                .with_mime_type("text/plain"),
        ),
        MultipartFormFileField::new(
            "file1",
            MultipartFile::new("Goodbye, world!")
                .with_name("goodbye_world.dat")
                .with_mime_type("application/octet-stream"),
        ),
    ]));

    let response: Response = Request::post(&bridge)
        .multipart_body(multipart)
        .to("upload")
        .send()
        .await
        .expect("http rest request error");

    assert_eq!(response.status_code().as_u16(), 200);
}
