//! Graphql example
//!
//! In this example there's the explanation on how to achieve a graphql call. This serialize a request
//! containing variables and post to graphql server, deserializing response. The response is the list
//! of all jobs matching given predicate ("backend-engineer").
//!
//! Run this to run the example:
//!
//! ```shell
//! cargo run --example graphql
//! ```

use serde::{Deserialize, Serialize};

use prima_bridge::prelude::*;
use prima_bridge::ParsedGraphqlResponse;

const URL: &str = "https://api.graphql.jobs/";
const QUERY: &str = "query($input:JobsInput!){jobs(input:$input) {\nid\n title\n applyUrl\n}}";

#[tokio::main]
async fn main() {
    let bridge: Bridge = Bridge::builder().build(URL.parse().unwrap());

    let slug: String = "backend-engineer".to_string();

    let response: Response = GraphQLRequest::new(&bridge, (QUERY, Some(JobsRequest::new(slug))))
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
