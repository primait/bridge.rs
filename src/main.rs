#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use httpmock::prelude::*;
    use prima_bridge::{prelude::DeliverableRequest, Bridge, GraphQLRequest};
    use reqwest::Url;

    #[tokio::test]
    async fn foo() {
        let path: &str = "/upload";

        let url: Url = Url::parse(&server.url(path)).unwrap();

        let bridge = Bridge::builder().build(url);
        GraphQLRequest::new_with_uploads(
            &bridge,
            ("asd", Some(())),
            HashMap::from([("carlo".to_string(), "ciao".as_bytes().to_vec())]),
        )
        .expect("Oops")
        .send()
        .await
        .expect("kek");
    }
}

fn main() {}
