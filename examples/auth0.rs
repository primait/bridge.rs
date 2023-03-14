//! Auth0 example
//!
//! This example is the same as examples/graphql.rs example. The only difference is that in this example
//! every request made by the bridge are authenticated with a JWT token as Bearer authentication.
//!
//! Before run the command to run this example be sure redis is running. Otherwise run this command:
//!
//! ```shell
//! docker run -p 6379:6379 --name bridge_redis -d redis
//! ```
//!
//! Export needed variables:
//! ```shell
//! export TOKEN_URL='https://{tenant}.auth0.com/oauth/token'
//! export JWKS_URL='https://{tenant}/.well-known/jwks.json'
//! export CLIENT_ID={client_id}
//! export CLIENT_SECRET={client_secret}
//! export AUDIENCE={audience}
//! ```
//!
//! And then to run this example execute:
//!
//! ```shell
//! cargo run --example auth0 --features auth0
//! ```

use prima_bridge::{prelude::*, ParsedGraphqlResponse};
use serde::{Deserialize, Serialize};

const URL: &str = "https://api.graphql.jobs/";
const QUERY: &str = "query($input:JobsInput!){jobs(input:$input) {\nid\n title\n applyUrl\n}}";

#[tokio::main]
async fn main() {
    let bridge: Bridge = Bridge::builder()
        .with_auth0(auth0::config())
        .await
        .build(URL.parse().unwrap());

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

mod auth0 {
    use std::time::Duration;

    use prima_bridge::auth0::{CacheType, Config, StalenessCheckPercentage};

    pub fn config() -> Config {
        use std::str::FromStr;

        use reqwest::Url;

        let token_url: String = std::env::var("TOKEN_URL").unwrap();
        let jwks_url: String = std::env::var("JWKS_URL").unwrap();
        let client_id: String = std::env::var("CLIENT_ID").unwrap();
        let client_secret: String = std::env::var("CLIENT_SECRET").unwrap();
        let audience: String = std::env::var("AUDIENCE").unwrap();

        Config {
            token_url: Url::from_str(token_url.as_str()).unwrap(),
            jwks_url: Url::from_str(jwks_url.as_str()).unwrap(),
            caller: "paperboy".to_string(),
            audience,
            cache_type: CacheType::Inmemory,
            token_encryption_key: "32char_long_token_encryption_key".to_string(),
            check_interval: Duration::from_secs(2),
            staleness_check_percentage: StalenessCheckPercentage::new(0.1, 0.5),
            client_id,
            client_secret,
        }
    }
}
