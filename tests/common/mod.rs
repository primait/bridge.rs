use mockito::{mock, BinaryBody, Matcher, Mock};
use reqwest::Url;

pub fn get_mock(status_code: usize, body: &str) -> (Mock, Url) {
    mock_with_path(status_code, body, "/")
}

pub fn mock_with_base_and_path(
    status_code: usize,
    body: &str,
    base: &str,
    path: &str,
) -> (Mock, Url) {
    let mock = mock("GET", format!("/{}/{}", base, path).as_str())
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .with_status(status_code)
        .with_body(body)
        .create();

    let base_url = format!("{}/{}", mockito::server_url(), base);
    (mock, Url::parse(base_url.as_str()).unwrap())
}

pub fn mock_with_path(status_code: usize, body: &str, path: &str) -> (Mock, Url) {
    let mock = mock("GET", path)
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .with_status(status_code)
        .with_body(body)
        .create();

    (mock, Url::parse(mockito::server_url().as_str()).unwrap())
}

pub fn mock_with_path_and_header(
    status_code: usize,
    body: &str,
    path: &str,
    header: (&str, &str),
) -> (Mock, Url) {
    let mock = mock("GET", path)
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .with_status(status_code)
        .with_header(header.0, header.1)
        .with_body(body)
        .create();

    (mock, Url::parse(mockito::server_url().as_str()).unwrap())
}

pub fn mock_with_raw_body_matcher(body: &str) -> (Mock, Url) {
    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_body(Matcher::Exact(body.to_owned()))
        .with_status(200)
        .create();
    (mock, Url::parse(mockito::server_url().as_str()).unwrap())
}

pub fn mock_with_header_matcher((name, value): (&str, &str)) -> (Mock, Url) {
    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_header(name, value)
        .with_status(200)
        .create();

    (mock, Url::parse(mockito::server_url().as_str()).unwrap())
}

pub fn mock_with_user_agent(user_agent: &str) -> (Mock, Url) {
    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_header("User-Agent", user_agent)
        .with_status(200)
        .create();

    (mock, Url::parse(mockito::server_url().as_str()).unwrap())
}

pub fn mock_with_json_body_matcher(json: serde_json::Value) -> (Mock, Url) {
    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_body(Matcher::Json(json))
        .with_status(200)
        .create();

    (mock, Url::parse(mockito::server_url().as_str()).unwrap())
}

pub fn mock_with_binary_body_matcher(body: &[u8]) -> (Mock, Url) {
    let mock = mock("GET", "/")
        .match_header(
            "x-request-id",
            Matcher::Regex(
                r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b".to_string(),
            ),
        )
        .match_body(Matcher::Binary(BinaryBody::from_bytes(body.to_vec())))
        .with_status(200)
        .with_body(body)
        .create();

    (mock, Url::parse(mockito::server_url().as_str()).unwrap())
}

pub fn create_auth0_mock() -> Mock {
    mock("GET", "/token")
        // .match_query(Matcher::UrlEncoded(
        //     "audience".to_string(),
        //     "test".to_string(),
        // ))
        .with_status(200)
        .with_body("{\"token\": \"abcdef\"}")
        .create()
}
