//! Graphql Multipart example
//!
//! In this example there's the explanation on how to achieve a graphql call including multipart files.
//! Sadly this example doesn't work (not able to find a live example with multipart uploads).
//!
//! Run this to run the example:
//!
//! ```shell
//! cargo run --example graphql_multipart
//! ```
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use prima_bridge::prelude::*;
use prima_bridge::{Multipart, MultipartFile, ParsedGraphqlResponse};

const URL: &str = "https://api.graphql.jobs/";
const QUERY: &str = "query($input:JobsInput!){jobs(input:$input) {\nid\n title\n applyUrl\n}}";

#[tokio::main]
async fn main() {
    let bridge: Bridge = Bridge::builder().build(URL.parse().unwrap());

    let bytes: Vec<u8> = vec![];

    let slug: String = "backend-engineer".to_string();

    let mut map: HashMap<String, Vec<MultipartFile>> = HashMap::new();
    let _ = map.insert(
        "variables.multi.files".to_string(),
        vec![
            MultipartFile::new(bytes.clone()).with_name("ciao1"),
            MultipartFile::new(bytes.clone()).with_name("ciao2"),
        ],
    );
    let _ = map.insert(
        "variables.multi.images".to_string(),
        vec![MultipartFile::new(bytes.clone()).with_name("ciao3")],
    );
    let multipart: Multipart = Multipart::multiple(map);

    let response: Response =
        GraphQLRequest::new_with_multipart(&bridge, (QUERY, Some(JobsRequest::new(slug))), multipart)
            .expect("Failed to create graphql request for query")
            .send()
            .await
            .expect("graphql request results in error");

    // The response is something like:
    //  {
    //      "data": {
    //          "jobs": [
    //              {
    //                  "id": "cjz1ipl9x009a0758hg68h7vy",
    //                  "title": "Senior Fullstack Engineer - Platform",
    //                  "applyUrl": "https://grnh.se/2d8f45d71"
    //              },
    //              ...
    //          ]
    //      }
    // }

    // Using `get_data` parse the response and select only desired field parsing into the type you
    // defined, eg. `PeopleResponse`.
    let response: ParsedGraphqlResponse<JobsResponse> = response
        .parse_graphql_response::<JobsResponse>()
        .expect("error parsing response");

    let response: JobsResponse = response.expect("response is not completed");

    println!(
        "The first job received from `{}` is {:?}",
        URL,
        response.jobs.first().unwrap()
    );
}

// Request
#[derive(Serialize, Debug)]
pub struct JobsRequest {
    pub input: JobsInput,
}

impl JobsRequest {
    pub fn new(slug: String) -> Self {
        Self {
            input: JobsInput { slug },
        }
    }
}

#[derive(Serialize, Debug)]
pub struct JobsInput {
    pub slug: String,
}

// Response

#[derive(Deserialize, Debug)]
pub struct JobsResponse {
    pub jobs: Vec<Job>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")] // This is needed in order to rename `apply_url` into `applyUrl`
pub struct Job {
    pub id: String,
    pub title: String,
    pub apply_url: String,
}
