use mockito::{mock, Matcher};
use prima_bridge::{Bridge, DeliverableRequest, GraphQLMultipart, GraphQLRequest, MultipartFile};
use std::collections::HashMap;
use std::error::Error;

const SINGLE_UPLOAD_QUERY: &str = r#"mutation($file: Upload!) {
    upload(file: $file) {
        id
    }
}"#;

#[tokio::test]
async fn multipart_graphql_single_file() -> Result<(), Box<dyn Error>> {
    let _mock = mock("POST", "/")
        .match_header("Content-Type", Matcher::Regex("^multipart/form-data".to_string()))
        .match_body(Matcher::AllOf(vec![
            Matcher::Regex(r#"Content-Disposition: form-data; name="operations"\s+.+"variables":\{"file":null\}"#.into()),
            Matcher::Regex(r#"Content-Disposition: form-data; name="map"\s+\{"0":\["variables\.file"\]\}"#.into()),
            Matcher::Regex(r#"Content-Disposition: form-data; name="0"; filename="howdy_world\.txt"\s+Content-Type: text/plain\s+Howdy!"#.into()),
        ]))
        .with_status(200)
        .with_body(r#"{"data": {}}"#)
        .create();

    let bridge = Bridge::builder().build(mockito::server_url().parse().unwrap());
    let multipart_body = GraphQLMultipart::single(
        "variables.file",
        MultipartFile::new("Howdy!")
            .with_name("howdy_world.txt")
            .with_mime_type("text/plain"),
    );
    let response =
        GraphQLRequest::new_with_multipart(&bridge, (SINGLE_UPLOAD_QUERY, Option::<()>::None), multipart_body)?
            .send()
            .await?;

    assert!(response.is_ok());
    Ok(())
}

const MULTIPLE_UPLOAD_QUERY: &str = r#"mutation($files: [Upload]!) {
    multiUpload(files: $files) {
        id
    }
}"#;

#[tokio::test]
async fn multipart_graphql_multiple_files() -> Result<(), Box<dyn Error>> {
    let _mock = mock("POST", "/")
        .match_header("Content-Type", Matcher::Regex("^multipart/form-data".to_string()))
        .match_body(Matcher::AllOf(vec![
            Matcher::Regex(
                r#"Content-Disposition: form-data; name="operations"\s+.+"variables":\{"files":\[null,null\]\}"#.into(),
            ),
            Matcher::Regex(r#"Content-Disposition: form-data; name="map""#.into()),
            Matcher::Regex(r#"\{.*"0":\["variables\.files\.0"\].*\}"#.into()),
            Matcher::Regex(r#"\{.*"1":\["variables\.files\.1"\].*\}"#.into()),
            Matcher::Regex(r#"Content-Disposition: form-data; name="0"; filename="howdy_world\.txt""#.into()),
            Matcher::Regex(r#"Content-Disposition: form-data; name="1"; filename="hello_world\.txt""#.into()),
        ]))
        .with_status(200)
        .with_body(r#"{"data": {}}"#)
        .create();

    let bridge = Bridge::builder().build(mockito::server_url().parse().unwrap());

    let files = vec![
        MultipartFile::new("Howdy!")
            .with_name("howdy_world.txt")
            .with_mime_type("text/plain"),
        MultipartFile::new("Hello!")
            .with_name("hello_world.txt")
            .with_mime_type("text/plain"),
    ];
    let multipart_body = GraphQLMultipart::multiple(HashMap::from([("files".to_string(), files)]));
    let response =
        GraphQLRequest::new_with_multipart(&bridge, (MULTIPLE_UPLOAD_QUERY, Option::<()>::None), multipart_body)?
            .send()
            .await?;

    assert!(response.is_ok());
    Ok(())
}
