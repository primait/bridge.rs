use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::multipart::{Form, Part};
use reqwest::{Method, Url};
use serde::Serialize;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::request::{Body, DeliverableRequest, DeliverableRequestBody, GraphQLBody, RequestType};
use crate::{BridgeClient, BridgeImpl, MultipartFile};

const VARIABLES: &str = "variables";
const ZERO: &str = "0";

/// The GraphQLRequest is a struct that represent a GraphQL request to be done with a [crate::Bridge].
#[allow(clippy::upper_case_acronyms)]
pub struct GraphQLRequest<'a, Client: BridgeClient> {
    id: Uuid,
    bridge: &'a BridgeImpl<Client>,
    body: Body,
    method: Method,
    timeout: Duration,
    path: Option<&'a str>,
    query_pairs: Vec<(&'a str, &'a str)>,
    ignore_status_code: bool,
    custom_headers: HeaderMap,
    multipart: Option<GraphQLMultipart>,
}

impl<'a, Client: BridgeClient> GraphQLRequest<'a, Client> {
    /// Creates a new GraphQLRequest
    pub fn new<S: Serialize>(
        bridge: &'a BridgeImpl<Client>,
        graphql_body: impl Into<GraphQLBody<S>>,
    ) -> PrimaBridgeResult<Self> {
        let mut custom_headers = HeaderMap::default();
        custom_headers.append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(Self {
            id: Uuid::new_v4(),
            bridge,
            body: serde_json::to_string(&graphql_body.into())?.into(),
            method: Method::POST,
            path: Default::default(),
            timeout: Duration::from_secs(60),
            query_pairs: Default::default(),
            ignore_status_code: Default::default(),
            custom_headers,
            multipart: None,
        })
    }

    pub fn new_with_multipart<S: Serialize>(
        bridge: &'a BridgeImpl<Client>,
        graphql_body: impl Into<GraphQLBody<S>>,
        multipart: GraphQLMultipart,
    ) -> PrimaBridgeResult<Self> {
        // No content-type here because Form set it at `multipart/form-data` with extra params for
        // disposition
        let json_body = serde_json::to_value(graphql_body.into())?;
        let body_with_injected_variables = match &multipart {
            GraphQLMultipart::Single(single) => {
                let path: VecDeque<&str> = single.path.split('.').collect();
                add_field(path, json_body, &Value::Null)?
            }
            GraphQLMultipart::Multiple(multiple) => {
                multiple.map.iter().try_fold(json_body, |accumulator, (path, files)| {
                    let filler = Value::Array(vec![Value::Null; files.len()]);
                    files.iter().try_fold(accumulator, |acc, _| {
                        let path_vec: VecDeque<&str> = path.split('.').collect();
                        add_field(path_vec, acc, &filler)
                    })
                })?
            }
        };

        Ok(Self {
            id: Uuid::new_v4(),
            bridge,
            body: serde_json::to_string(&body_with_injected_variables)?.into(),
            method: Method::POST,
            path: Default::default(),
            timeout: Duration::from_secs(60),
            query_pairs: Default::default(),
            ignore_status_code: Default::default(),
            custom_headers: HeaderMap::new(),
            multipart: Some(multipart),
        })
    }
}

#[async_trait]
impl<'a, Client: BridgeClient> DeliverableRequest<'a> for GraphQLRequest<'a, Client> {
    type Client = Client;
    fn raw_body(self, body: impl Into<Body>) -> Self {
        Self {
            body: body.into(),
            ..self
        }
    }

    fn json_body<B: Serialize>(self, body: &B) -> PrimaBridgeResult<Self> {
        Ok(Self {
            body: serde_json::to_string(body)?.into(),
            ..self
        })
    }

    fn method(self, method: Method) -> Self {
        Self { method, ..self }
    }

    fn to(self, path: &'a str) -> Self {
        Self {
            path: Some(path),
            ..self
        }
    }

    fn ignore_status_code(self) -> Self {
        Self {
            ignore_status_code: true,
            ..self
        }
    }

    fn set_timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }

    fn get_timeout(&self) -> Duration {
        self.timeout
    }

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_bridge(&self) -> &BridgeImpl<Client> {
        self.bridge
    }

    fn get_path(&self) -> Option<&str> {
        self.path
    }

    fn endpoint(&self) -> Url {
        self.bridge.endpoint.clone()
    }

    fn get_query_pairs(&self) -> &[(&'a str, &'a str)] {
        self.query_pairs.as_slice()
    }

    fn get_query_pairs_mut(&mut self) -> &mut Vec<(&'a str, &'a str)> {
        &mut self.query_pairs
    }

    fn get_ignore_status_code(&self) -> bool {
        self.ignore_status_code
    }

    fn get_method(&self) -> Method {
        self.method.clone()
    }

    fn get_custom_headers(&self) -> &HeaderMap {
        &self.custom_headers
    }

    fn get_custom_headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.custom_headers
    }

    #[cfg(feature = "auth0")]
    fn get_auth0(&self) -> &Option<crate::auth0::RefreshingToken> {
        &self.bridge.auth0_opt
    }

    fn into_body(self) -> PrimaBridgeResult<DeliverableRequestBody> {
        Ok(match self.multipart {
            Some(multipart) => DeliverableRequestBody::Multipart(multipart.into_form(self.body)?),
            None => DeliverableRequestBody::RawBody(self.body),
        })
    }

    fn get_body(&self) -> Option<&[u8]> {
        self.body.as_bytes()
    }

    fn get_request_type(&self) -> RequestType {
        RequestType::GraphQL
    }
}

/// A [GraphQLRequest] multipart body as according to [this specification](https://github.com/jaydenseric/graphql-multipart-request-spec).
///
/// Only files can be included in the multipart body.  
/// It can either be `Single` (one file) or `Multiple` (multiple files).
pub enum GraphQLMultipart {
    Single(#[doc(hidden)] Single),
    Multiple(#[doc(hidden)] Multiple),
}

impl GraphQLMultipart {
    /// Creates a GraphQL multipart body containing a single file.
    ///
    /// The `path` parameter is the path to the field in the GraphQL `variables` object
    /// that will act as a placeholder for the uploaded file.  
    /// The field will be automatically injected into the `variables` object: you don't need to add it
    /// to your existing variables struct.
    pub fn single(path: impl Into<String>, file: MultipartFile) -> Self {
        Self::Single(Single::new(path.into(), file))
    }

    /// Creates a GraphQL multipart body containing a list of files or multiple lists of files.
    ///
    /// Each key in the hash map is the path to the array field in the GraphQL `variables`
    /// object that will act as a placeholder for the list of files corresponding to that key.  
    /// The fields will be automatically injected into the `variables` object: you don't need to add them
    /// to your existing variables struct.
    pub fn multiple(map: HashMap<impl Into<String>, Vec<MultipartFile>>) -> Self {
        Self::Multiple(Multiple::from_map(map))
    }

    pub(crate) fn into_form(self, body: Body) -> PrimaBridgeResult<Form> {
        let mut form: Form = Form::new();
        form = form.part("operations", Part::stream(body.inner));

        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        match self {
            Self::Single(single) => {
                map.insert(ZERO.to_string(), vec![single.path.to_string()]);
                form = form.part(ZERO.to_string(), single.file.into_part()?);
            }
            Self::Multiple(multiple) => {
                let mut index = 0;
                for (path, files) in multiple.map.into_iter() {
                    for (id, file) in files.into_iter().enumerate() {
                        map.insert(index.to_string(), vec![format!("{}.{}", path, id)]);
                        form = form.part(index.to_string(), file.into_part()?);
                        index += 1;
                    }
                }
            }
        }

        let map_string: String =
            serde_json::to_string(&map).expect("Internal error while to serializing form field `map`");
        form = form.text("map", map_string);
        Ok(form)
    }
}

pub struct Single {
    path: String,
    file: MultipartFile,
}

impl Single {
    pub fn new(path: String, file: MultipartFile) -> Self {
        Self {
            path: with_variables_prefix(path),
            file,
        }
    }
}

pub struct Multiple {
    map: HashMap<String, Vec<MultipartFile>>,
}

impl Multiple {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn from_map(map: HashMap<impl Into<String>, Vec<MultipartFile>>) -> Self {
        Self {
            map: map
                .into_iter()
                .map(|(path, files)| (with_variables_prefix(path.into()), files))
                .collect(),
        }
    }
}

impl Default for Multiple {
    fn default() -> Self {
        Self::new()
    }
}

fn with_variables_prefix(path: String) -> String {
    if path.starts_with(VARIABLES) {
        path
    } else {
        format!("{}.{}", VARIABLES, path)
    }
}

fn add_field(mut path_segments: VecDeque<&str>, json: Value, field_value: &Value) -> Result<Value, PrimaBridgeError> {
    Ok(match path_segments.pop_front() {
        Some(segment) => match json {
            Value::Null => craft_field(path_segments, segment, Map::new(), Value::Null, field_value)?,
            Value::Object(map) => {
                let inner_value: Value = map.get(segment).cloned().unwrap_or(Value::Null);
                craft_field(path_segments, segment, map, inner_value, field_value)?
            }
            _ => return Err(PrimaBridgeError::MalformedVariables),
        },
        None => field_value.to_owned(),
    })
}

fn craft_field(
    paths: VecDeque<&str>,
    segment: &str,
    mut map: Map<String, Value>,
    inner_value: Value,
    field_value: &Value,
) -> PrimaBridgeResult<Value> {
    let value: Value = add_field(paths, inner_value, field_value)?;
    map.insert(segment.to_string(), value);
    Ok(Value::Object(map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bridge;
    use serde::Deserialize;
    use serde_json::json;
    use std::str::FromStr;

    #[derive(Deserialize)]
    pub struct VariablesSingle {
        #[allow(dead_code)]
        pub input: InputSingle,
    }

    #[derive(Deserialize)]
    pub struct InputSingle {
        #[allow(dead_code)]
        pub file: (),
    }

    #[derive(Deserialize)]
    pub struct VariablesMulti {
        #[allow(dead_code)]
        pub input: InputMulti,
    }

    #[allow(dead_code)]
    #[derive(Deserialize)]
    pub struct InputMulti {
        pub files: Vec<()>,
        pub images: Vec<()>,
    }

    #[test]
    fn multipart_with_single_file_with_explicit_variables_mapping() {
        let fake_url: Url = Url::from_str("http://prima.it").unwrap();
        let bridge: Bridge = Bridge::builder().build(fake_url);
        let multipart: GraphQLMultipart = get_single_file_multipart();

        let variables: Value = get_single_file_multipart_value();

        let req = GraphQLRequest::new_with_multipart(&bridge, ("query", Some(variables.clone())), multipart).unwrap();

        let body_str = req.body.as_str().unwrap();
        let graphql_body: GraphQLBody<Value> = serde_json::from_str(body_str.as_ref()).unwrap();
        assert_eq!(graphql_body.variables.unwrap(), variables);

        let parsed_graphql_body: Result<GraphQLBody<VariablesSingle>, serde_json::Error> =
            serde_json::from_str(body_str.as_ref());

        assert!(parsed_graphql_body.is_ok());
    }

    #[test]
    fn multipart_with_single_file_without_explicit_variables_mapping() {
        let fake_url: Url = Url::from_str("http://prima.it").unwrap();
        let bridge: Bridge = Bridge::builder().build(fake_url);
        let multipart: GraphQLMultipart = get_single_file_multipart();

        let variables: Value = Value::Null;

        let req = GraphQLRequest::new_with_multipart(&bridge, ("query", Some(variables)), multipart).unwrap();

        let body_str = req.body.as_str().unwrap();

        let graphql_body: GraphQLBody<Value> = serde_json::from_str(body_str.as_ref()).unwrap();

        let expectation: Value = get_single_file_multipart_value();
        assert_eq!(graphql_body.variables.unwrap(), expectation);

        let parsed_graphql_body: Result<GraphQLBody<VariablesSingle>, serde_json::Error> =
            serde_json::from_str(body_str.as_ref());

        assert!(parsed_graphql_body.is_ok());
    }

    #[test]
    fn multipart_with_single_file_with_explicit_variables_mapping_with_wrong_value() {
        let fake_url: Url = Url::from_str("http://prima.it").unwrap();
        let bridge: Bridge = Bridge::builder().build(fake_url);
        let multipart: GraphQLMultipart = get_single_file_multipart();

        let variables: Value = json!({"input": {"file": Value::String("ciao".to_string())}});

        let req = GraphQLRequest::new_with_multipart(&bridge, ("query", Some(variables)), multipart).unwrap();

        let body_str = req.body.as_str().unwrap();
        let graphql_body: GraphQLBody<Value> = serde_json::from_str(body_str.as_ref()).unwrap();

        let expectation: Value = get_single_file_multipart_value();
        assert_eq!(graphql_body.variables.unwrap(), expectation);

        let parsed_graphql_body: Result<GraphQLBody<VariablesSingle>, serde_json::Error> =
            serde_json::from_str(body_str.as_ref());

        assert!(parsed_graphql_body.is_ok());
    }

    #[test]
    fn multipart_with_single_file_with_explicit_variables_mapping_with_additional_fields() {
        let fake_url: Url = Url::from_str("http://prima.it").unwrap();
        let bridge: Bridge = Bridge::builder().build(fake_url);
        let multipart: GraphQLMultipart = get_single_file_multipart();

        let variables: Value = json!({
            "other":{"whatever": Value::String("hello".to_string())},
            "input": {"file": Value::String("ciao".to_string()), "desc": Value::String("desc".to_string())}
        });

        let req = GraphQLRequest::new_with_multipart(&bridge, ("query", Some(variables)), multipart).unwrap();

        let body_str = req.body.as_str().unwrap();
        let graphql_body: GraphQLBody<Value> = serde_json::from_str(body_str.as_ref()).unwrap();

        let expectation: Value = json!({
            "other":{"whatever": Value::String("hello".to_string())},
            "input": {"file": Value::Null, "desc": Value::String("desc".to_string())}
        });
        assert_eq!(graphql_body.variables.unwrap(), expectation);

        let parsed_graphql_body: Result<GraphQLBody<VariablesSingle>, serde_json::Error> =
            serde_json::from_str(body_str.as_ref());

        assert!(parsed_graphql_body.is_ok());
    }

    #[test]
    fn multipart_with_multiple_files_with_explicit_variables_mapping() {
        let fake_url: Url = Url::from_str("http://prima.it").unwrap();
        let bridge: Bridge = Bridge::builder().build(fake_url);
        let multipart: GraphQLMultipart = get_multi_files_multipart();

        let variables: Value = get_multi_files_multipart_value();

        let req = GraphQLRequest::new_with_multipart(&bridge, ("query", Some(variables.clone())), multipart).unwrap();

        let body_str = req.body.as_str().unwrap();
        let graphql_body: GraphQLBody<Value> = serde_json::from_str(body_str.as_ref()).unwrap();

        assert_eq!(graphql_body.variables.unwrap(), variables);

        let parsed_graphql_body: Result<GraphQLBody<VariablesMulti>, serde_json::Error> =
            serde_json::from_str(body_str.as_ref());

        assert!(parsed_graphql_body.is_ok());
    }

    #[test]
    fn multipart_with_multiple_files_with_explicit_variables_mapping_with_additional_fields() {
        let fake_url: Url = Url::from_str("http://prima.it").unwrap();
        let bridge: Bridge = Bridge::builder().build(fake_url);
        let multipart: GraphQLMultipart = get_multi_files_multipart();

        let variables: Value = get_multi_files_multipart_value();

        let req = GraphQLRequest::new_with_multipart(&bridge, ("query", Some(variables.clone())), multipart).unwrap();

        let body_str = req.body.as_str().unwrap();
        let graphql_body: GraphQLBody<Value> = serde_json::from_str(body_str.as_ref()).unwrap();

        assert_eq!(graphql_body.variables.unwrap(), variables);

        let parsed_graphql_body: Result<GraphQLBody<VariablesMulti>, serde_json::Error> =
            serde_json::from_str(body_str.as_ref());

        assert!(parsed_graphql_body.is_ok());
    }

    #[test]
    fn multipart_with_multiple_files_with_partial_explicit_variables_mapping() {
        let fake_url: Url = Url::from_str("http://prima.it").unwrap();
        let bridge: Bridge = Bridge::builder().build(fake_url);
        let multipart: GraphQLMultipart = get_multi_files_multipart();

        let variables: Value = json!({"input": {"files": Value::Array(vec![Value::Null, Value::Null])}});

        let req = GraphQLRequest::new_with_multipart(&bridge, ("query", Some(variables)), multipart).unwrap();

        let body_str = req.body.as_str().unwrap();
        let graphql_body: GraphQLBody<Value> = serde_json::from_str(body_str.as_ref()).unwrap();

        let expectation: Value = get_multi_files_multipart_value();
        assert_eq!(graphql_body.variables.unwrap(), expectation);

        let parsed_graphql_body: Result<GraphQLBody<VariablesMulti>, serde_json::Error> =
            serde_json::from_str(body_str.as_ref());

        assert!(parsed_graphql_body.is_ok());
    }

    #[test]
    fn multipart_with_multiple_files_with_partial_explicit_variables_mapping_with_wrong_values() {
        let fake_url: Url = Url::from_str("http://prima.it").unwrap();
        let bridge: Bridge = Bridge::builder().build(fake_url);
        let multipart: GraphQLMultipart = get_multi_files_multipart();

        let variables: Value = json!({"input": {"files": Value::String("ciao".to_string())}});

        let req = GraphQLRequest::new_with_multipart(&bridge, ("query", Some(variables)), multipart).unwrap();

        let body_str = req.body.as_str().unwrap();
        let graphql_body: GraphQLBody<Value> = serde_json::from_str(body_str.as_ref()).unwrap();

        let expectation: Value = get_multi_files_multipart_value();
        assert_eq!(graphql_body.variables.unwrap(), expectation);

        let parsed_graphql_body: Result<GraphQLBody<VariablesMulti>, serde_json::Error> =
            serde_json::from_str(body_str.as_ref());

        assert!(parsed_graphql_body.is_ok());
    }

    #[test]
    fn multipart_with_multiple_files_without_explicit_variables_mapping() {
        let fake_url: Url = Url::from_str("http://prima.it").unwrap();
        let bridge: Bridge = Bridge::builder().build(fake_url);
        let multipart: GraphQLMultipart = get_multi_files_multipart();

        let variables: Value = Value::Null;

        let req = GraphQLRequest::new_with_multipart(&bridge, ("query", Some(variables)), multipart).unwrap();

        let body_str = req.body.as_str().unwrap();
        let graphql_body: GraphQLBody<Value> = serde_json::from_str(body_str.as_ref()).unwrap();

        let expectation: Value = get_multi_files_multipart_value();
        assert_eq!(graphql_body.variables.unwrap(), expectation);

        let parsed_graphql_body: Result<GraphQLBody<VariablesMulti>, serde_json::Error> =
            serde_json::from_str(body_str.as_ref());

        assert!(parsed_graphql_body.is_ok());
    }

    fn get_single_file_multipart_value() -> Value {
        json!({"input": {"file": Value::Null}})
    }

    fn get_multi_files_multipart_value() -> Value {
        json!({"input": {"files": Value::Array(vec![Value::Null, Value::Null]), "images": Value::Array(vec![Value::Null])}})
    }

    fn get_single_file_multipart() -> GraphQLMultipart {
        let path: &str = "input.file";
        let file: MultipartFile = MultipartFile::new(vec![]).with_name("ciao1");
        GraphQLMultipart::single(path, file)
    }

    fn get_multi_files_multipart() -> GraphQLMultipart {
        let mut map: HashMap<String, Vec<MultipartFile>> = HashMap::new();
        let _ = map.insert(
            "variables.input.files".to_string(),
            vec![
                MultipartFile::new(vec![]).with_name("ciao1"),
                MultipartFile::new(vec![]).with_name("ciao2"),
            ],
        );
        let _ = map.insert(
            "input.images".to_string(),
            vec![MultipartFile::new(vec![]).with_name("ciao3")],
        );
        GraphQLMultipart::multiple(map)
    }
}
