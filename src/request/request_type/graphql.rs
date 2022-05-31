use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::multipart::{Form, Part};
use reqwest::{Method, Url};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::errors::{PrimaBridgeError, PrimaBridgeResult};
use crate::request::{Body, DeliverableRequest, GraphQLBody, RequestType};
use crate::Bridge;

const DEFAULT_FILENAME: &str = "default";
const ZERO: &str = "0";

/// The GraphQLRequest is a struct that represent a GraphQL request to be done with the [Bridge](./../struct.Bridge.html)
#[allow(clippy::upper_case_acronyms)]
pub struct GraphQLRequest<'a> {
    id: Uuid,
    bridge: &'a Bridge,
    body: Body,
    method: Method,
    timeout: Duration,
    path: Option<&'a str>,
    query_pairs: Vec<(&'a str, &'a str)>,
    ignore_status_code: bool,
    custom_headers: HeaderMap,
    multipart: Option<Multipart>,
}

impl<'a> GraphQLRequest<'a> {
    /// creates a new GraphQLRequest
    pub fn new<S: Serialize>(
        bridge: &'a Bridge,
        graphql_body: impl Into<GraphQLBody<S>>,
    ) -> PrimaBridgeResult<Self> {
        let mut custom_headers = HeaderMap::default();
        custom_headers.append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(Self {
            id: Uuid::new_v4(),
            bridge,
            body: serde_json::to_string(&graphql_body.into())?.try_into()?,
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
        bridge: &'a Bridge,
        graphql_body: impl Into<GraphQLBody<S>>,
        multipart: Multipart,
    ) -> PrimaBridgeResult<Self> {
        // No content-type here because Form set it at `multipart/form-data` with extra params for
        // disposition
        let body: GraphQLBody<S> = graphql_body.into();
        if body.variables.is_none() {
            return Err(PrimaBridgeError::EmptyVariablesInMultipartRequest);
        }

        let mut body_as_value: Value = serde_json::to_value(&body)?;

        match &multipart {
            Multipart::Single(single) => {
                let mut value_opt: Option<&mut Value> = Some(&mut body_as_value);
                for path in single.path.split('.').collect::<Vec<&str>>() {
                    value_opt = value_opt.and_then(|a| a.get_mut(path));
                }

                match value_opt {
                    Some(value) => *value = json!(Value::Null),
                    None => {
                        return Err(PrimaBridgeError::MultipartFilePathNotFound(
                            single.path.clone(),
                        ))
                    }
                };
            }
            Multipart::Multiple(multiple) => {
                for (path, files) in &multiple.map {
                    let mut value_opt: Option<&mut Value> = Some(&mut body_as_value);
                    for path in path.split('.').collect::<Vec<&str>>() {
                        value_opt = value_opt.and_then(|a| a.get_mut(path));
                    }

                    match value_opt {
                        Some(value) => {
                            *value =
                                json!(Value::Array(files.iter().map(|_| Value::Null).collect()))
                        }
                        None => {
                            return Err(PrimaBridgeError::MultipartFilePathNotFound(path.clone()))
                        }
                    };
                }
            }
        };

        Ok(Self {
            id: Uuid::new_v4(),
            bridge,
            body: serde_json::to_string(&body_as_value)?.try_into()?,
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
impl<'a> DeliverableRequest<'a> for GraphQLRequest<'a> {
    fn raw_body(self, body: impl Into<Body>) -> Self {
        Self {
            body: body.into(),
            ..self
        }
    }

    fn json_body<B: Serialize>(self, body: &B) -> PrimaBridgeResult<Self> {
        Ok(Self {
            body: serde_json::to_string(body)?.try_into()?,
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

    fn set_custom_headers(self, headers: HeaderMap) -> Self {
        Self {
            custom_headers: headers,
            ..self
        }
    }

    fn set_query_pairs(self, query_pairs: Vec<(&'a str, &'a str)>) -> Self {
        Self {
            query_pairs,
            ..self
        }
    }

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_bridge(&self) -> &Bridge {
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

    fn get_ignore_status_code(&self) -> bool {
        self.ignore_status_code
    }

    fn get_method(&self) -> Method {
        self.method.clone()
    }

    fn get_custom_headers(&self) -> HeaderMap {
        self.custom_headers.clone()
    }

    #[cfg(feature = "auth0")]
    fn get_auth0(&self) -> &Option<crate::auth0::Auth0> {
        &self.bridge.auth0_opt
    }

    fn get_body(&self) -> Vec<u8> {
        self.body.clone().into()
    }

    fn get_request_type(&self) -> RequestType {
        RequestType::GraphQL
    }

    fn get_form(&self) -> Option<Form> {
        match &self.multipart {
            None => None,
            Some(multipart) => {
                let mut form: Form = Form::new();
                let mut map: HashMap<String, Vec<String>> = HashMap::<String, Vec<String>>::new();
                form = form.text("operations", String::from(&self.body));

                match multipart {
                    Multipart::Single(single) => {
                        map.insert(ZERO.to_string(), vec![single.path.to_string()]);
                        let part: Part =
                            Part::bytes(single.file.bytes().to_vec()).file_name(single.file.name());
                        form = form.part(ZERO.to_string(), part);
                    }
                    Multipart::Multiple(multiple) => {
                        let mut index = 0;
                        for (path, files) in multiple.map.iter() {
                            for (id, file) in files.iter().enumerate() {
                                map.insert(index.to_string(), vec![format!("{}.{}", path, id)]);
                                let part: Part =
                                    Part::bytes(file.bytes().to_vec()).file_name(file.name());
                                form = form.part(index.to_string(), part);
                                index += 1;
                            }
                        }
                    }
                }

                form = form.text("map", serde_json::to_string(&map).unwrap());
                Some(form)
            }
        }
    }
}

// The path in query variable. Eg: if query/mutation has a param named files (representing the
// multipart upload) this should be something like `variables.files`
pub enum Multipart {
    Single(Single),
    Multiple(Multiple),
}

impl Multipart {
    pub fn single(path: impl Into<String>, file: MultipartFile) -> Self {
        Self::Single(Single::new(path.into(), file))
    }

    pub fn multiple(map: HashMap<String, Vec<MultipartFile>>) -> Self {
        Self::Multiple(Multiple { map })
    }
}

pub struct Single {
    path: String,
    file: MultipartFile,
}

impl Single {
    pub fn new(path: String, file: MultipartFile) -> Self {
        Self { path, file }
    }
}

pub struct Multiple {
    map: HashMap<String, Vec<MultipartFile>>,
}

impl Multiple {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add_file(mut self, path: String, file: MultipartFile) -> Self {
        match self.map.get(&path) {
            Some(files) => {
                let mut files = files.to_vec();
                files.push(file);
                self.map.insert(path, files);
            }
            None => {
                self.map.insert(path, vec![file]);
            }
        };
        self
    }
}

impl Default for Multiple {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MultipartFile {
    bytes: Vec<u8>,
    name_opt: Option<String>,
    mime_type_opt: Option<String>,
}

impl MultipartFile {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            name_opt: None,
            mime_type_opt: None,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn name(&self) -> String {
        match &self.name_opt {
            Some(name) => name.clone(),
            None => DEFAULT_FILENAME.to_string(),
        }
    }

    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name_opt: Some(name.into()),
            ..self
        }
    }

    pub fn mime_type(self) -> Option<String> {
        self.mime_type_opt
    }

    pub fn with_mime_type(self, mime_type: impl Into<String>) -> Self {
        Self {
            mime_type_opt: Some(mime_type.into()),
            ..self
        }
    }
}
