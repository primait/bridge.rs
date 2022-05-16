#[cfg(test)]
mod tests {
    use std::{str::FromStr, collections::HashMap};
    
    use prima_bridge::{GraphQLRequest, Bridge, prelude::DeliverableRequest};
    use reqwest::Url;

    #[tokio::test]
    async fn foo() {
        let bridge = Bridge::builder()
            .build(Url::from_str("http://localhost:6112").unwrap());
        GraphQLRequest::new_with_uploads(
            &bridge,
            ("asd", Some(())),
            HashMap::from([
                ("carlo".to_string(), "ciao".as_bytes().to_vec())
            ]),
        ).expect("Oops")
        .send()
        .await
        .expect("kek");
    }
}

fn main() {}
