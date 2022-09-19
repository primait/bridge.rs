use prima_bridge::prelude::*;
use prima_bridge::{MultipartFile, MultipartFormFileField, RestMultipart};
use reqwest::Method;
use std::collections::HashSet;
use std::error::Error;

#[tokio::test]
async fn multipart_rest_single_file() -> Result<(), Box<dyn Error>> {
    let _mock = mockito::mock("POST", "/")
        .with_status(200)
        .with_body("{\"hello\": \"world!\"}")
        .match_header("Content-Type", mockito::Matcher::Regex(r#"^multipart/form-data"#.to_string()))
        .match_body(mockito::Matcher::Regex(
            r#"Content-Disposition: form-data; name="my_single_file"; filename="hello_world\.txt"\s+Content-Type: text/plain\s+Hello, world!"#.to_string()
        ))
        .create();

    let bridge = Bridge::builder().build(mockito::server_url().parse().unwrap());

    let multipart = RestMultipart::single(
        "my_single_file",
        MultipartFile::new(b"Hello, world!".to_vec())
            .with_name("hello_world.txt")
            .with_mime_type("text/plain"),
    );

    let result: String = RestRequest::new_with_multipart(&bridge, multipart)
        .method(Method::POST)
        .send()
        .await?
        .get_data(&["hello"])?;
    assert_eq!(result, "world!");

    Ok(())
}

#[tokio::test]
async fn multipart_rest_multi_file() -> Result<(), Box<dyn Error>> {
    // Because a HashSet is used in RestMultipart::multiple, the order of each file is undefined, so the
    // matching Regex must be able to match the files in either order.
    let re_first_file = r#"Content-Disposition: form-data; name="first_file"; filename="hello_world\.txt"\s+Content-Type: text/plain\s+Hello, world!"#;
    let re_second_file = r#"Content-Disposition: form-data; name="second_file"; filename="goodbye_world\.dat"\s+Content-Type: application/octet-stream\s+Goodbye, world!"#;
    let re_body = format!(
        // This will generate a regular expression that matches re_first_file...re_second_file OR re_second_file...re_first_file
        r#"{re_first_file}[\s\S]+\S[\s\S]+{re_second_file}|{re_second_file}[\s\S]+\S[\s\S]+{re_first_file}"#,
        re_first_file = re_first_file,
        re_second_file = re_second_file
    );

    let _mock = mockito::mock("POST", "/")
        .with_status(200)
        .with_body("{\"hello\": \"world!\"}")
        .match_header(
            "Content-Type",
            mockito::Matcher::Regex(r#"^multipart/form-data"#.to_string()),
        )
        .match_body(mockito::Matcher::Regex(re_body))
        .create();

    let bridge = Bridge::builder().build(mockito::server_url().parse().unwrap());

    let multipart = RestMultipart::multiple(HashSet::from_iter(
        [
            MultipartFormFileField::new(
                "first_file",
                MultipartFile::new(b"Hello, world!".to_vec())
                    .with_name("hello_world.txt")
                    .with_mime_type("text/plain"),
            ),
            MultipartFormFileField::new(
                "second_file",
                MultipartFile::new(b"Goodbye, world!".to_vec())
                    .with_name("goodbye_world.dat")
                    .with_mime_type("application/octet-stream"),
            ),
        ]
        .into_iter(),
    ));

    let result: String = RestRequest::new_with_multipart(&bridge, multipart)
        .method(Method::POST)
        .send()
        .await?
        .get_data(&["hello"])?;
    assert_eq!(result, "world!");

    Ok(())
}

#[tokio::test]
async fn multipart_rest_single_file_stream() -> Result<(), Box<dyn Error>> {
    let _mock = mockito::mock("POST", "/")
        .with_status(200)
        .with_body("{\"hello\": \"world!\"}")
        .match_header("Content-Type", mockito::Matcher::Regex(r#"^multipart/form-data"#.to_string()))
        .match_body(mockito::Matcher::Regex(r#"Content-Disposition: form-data; name="my_streamed_file"; filename="howdy_world\.txt"\s+Content-Type: text/plain\s+Howdy, world!"#.to_string()))
        .create();

    let bridge = Bridge::builder().build(mockito::server_url().parse().unwrap());

    let multipart = RestMultipart::single(
        "my_streamed_file",
        MultipartFile::new(tokio::fs::File::open("tests/resources/howdy_world.txt").await.unwrap())
            .with_name("howdy_world.txt")
            .with_mime_type("text/plain"),
    );

    let result: String = RestRequest::new_with_multipart(&bridge, multipart)
        .method(Method::POST)
        .send()
        .await?
        .get_data(&["hello"])?;
    assert_eq!(result, "world!");

    Ok(())
}
