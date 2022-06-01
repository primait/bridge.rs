//! Rest example
//!
//! In this example there's the explanation on how to achieve a rest call to an endpoint and parsing
//! the response. The response is the person fetched from persons api with id=1.
//!
//! Run this to run the example:
//!
//! ```shell
//! cargo run --example graphql_multipart
//! ```
use serde::Deserialize;

use prima_bridge::prelude::*;

const URL: &str = "https://swapi.dev/api";

#[tokio::main]
async fn main() {
    let bridge: Bridge = Bridge::builder().build(URL.parse().unwrap());

    let response: Response = Request::get(&bridge).to("people/1").send().await.expect(x);

    // The response is something like:
    // {
    //     "name": "Luke Skywalker",
    //     "height": "172",
    //     "mass": "77",
    //     "hair_color": "blond",
    //     "skin_color": "fair",
    //     "eye_color": "blue",
    //     ...
    // }

    // Using `get_data` parse the response and select only desired field parsing into the type you
    // defined, eg. `PeopleResponse`.
    let response: PeopleResponse = response.get_data(&[]).expect("error while fetching data");

    println!("The name got from `{}` is {}", URL, response.name);
}

#[derive(Deserialize, Debug)]
pub struct PeopleResponse {
    name: String,
}
