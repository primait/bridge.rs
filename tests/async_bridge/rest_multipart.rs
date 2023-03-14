use std::{collections::HashSet, error::Error};

use mockito::{Matcher, Server};
use prima_bridge::{prelude::*, MultipartFile, MultipartFormFileField, RestMultipart};

#[tokio::test]
async fn multipart_rest_single_file() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new_async().await;

    let _mock = server.mock("POST", "/")
        .with_status(200)
        .with_body("{\"hello\": \"world!\"}")
        .match_header("Content-Type", Matcher::Regex(r#"^multipart/form-data"#.to_string()))
        .match_body(Matcher::Regex(
            r#"Content-Disposition: form-data; name="my_single_file"; filename="hello_world\.txt"\s+Content-Type: text/plain\s+Hello, world!"#.to_string()
        ))
        .create_async().await;

    let bridge = Bridge::builder().build(server.url().parse().unwrap());

    let multipart = RestMultipart::single(
        "my_single_file",
        MultipartFile::new(b"Hello, world!".to_vec())
            .with_name("hello_world.txt")
            .with_mime_type("text/plain"),
    );

    let result: String = Request::post(&bridge)
        .multipart_body(multipart)
        .send()
        .await?
        .get_data(&["hello"])?;
    assert_eq!(result, "world!");

    Ok(())
}

#[tokio::test]
async fn multipart_rest_multi_file() -> Result<(), Box<dyn Error>> {
    let re_first_file = r#"Content-Disposition: form-data; name="first_file"; filename="hello_world\.txt"\s+Content-Type: text/plain\s+Hello, world!"#;
    let re_second_file = r#"Content-Disposition: form-data; name="second_file"; filename="goodbye_world\.dat"\s+Content-Type: application/octet-stream\s+Goodbye, world!"#;

    let mut server = Server::new_async().await;

    let _mock = server
        .mock("POST", "/")
        .with_status(200)
        .with_body("{\"hello\": \"world!\"}")
        .match_header("Content-Type", Matcher::Regex(r#"^multipart/form-data"#.to_string()))
        // Because a HashSet is used in RestMultipart::multiple, the order of each file is undefined,
        // so we must be able to match the files in either order.
        .match_body(Matcher::AllOf(vec![
            Matcher::Regex(re_first_file.to_string()),
            Matcher::Regex(re_second_file.to_string()),
        ]))
        .create_async()
        .await;

    let bridge = Bridge::builder().build(server.url().parse().unwrap());

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

    let result: String = Request::post(&bridge)
        .multipart_body(multipart)
        .send()
        .await?
        .get_data(&["hello"])?;
    assert_eq!(result, "world!");

    Ok(())
}

#[tokio::test]
async fn multipart_rest_single_file_stream() {
    let mut server = Server::new_async().await;

    let _mock = server.mock("POST", "/")
        .with_status(200)
        .with_body("{\"hello\": \"world!\"}")
        .match_header("Content-Type", Matcher::Regex(r#"^multipart/form-data"#.to_string()))
        .match_body(Matcher::Regex(r#"Content-Disposition: form-data; name="my_streamed_file"; filename="howdy_world\.txt"\s+Content-Type: text/plain\s+Howdy, world!"#.to_string()))
        .create_async().await;

    let bridge = Bridge::builder().build(server.url().parse().unwrap());

    let multipart = RestMultipart::single(
        "my_streamed_file",
        MultipartFile::new(tokio::fs::File::open("tests/resources/howdy_world.txt").await.unwrap())
            .with_name("howdy_world.txt")
            .with_mime_type("text/plain"),
    );

    let result: String = Request::post(&bridge)
        .multipart_body(multipart)
        .send()
        .await
        .unwrap()
        .get_data(&["hello"])
        .unwrap();
    assert_eq!(result, "world!");
}
