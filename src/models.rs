use std::fmt::Debug;
use std::str::FromStr;

use crate::commands::command::Command;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pipeline {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub engine_version: String,

    /// Optional environment variables
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<indexmap::IndexMap<String, serde_json::Value>>,

    /// List of pipeline stages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipe: Option<Vec<Stage>>,

    /// Map of task names to their definitions ; expected to be used in jsonref context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<indexmap::IndexMap<String, Task>>,

    /// Map of schema names to their definitions ; expected to be used in jsonref context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<std::collections::HashMap<String, JsonSchema>>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub name: String,
    /// List of dependencies for this stage
    pub deps: Vec<Task>,
    /// Whether `Task`s in this stage should run sequentially
    pub sequential: bool,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub commands: Vec<Command>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<JsonSchema>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<JsonSchema>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<indexmap::IndexMap<String, serde_json::Value>>,
}

pub type JsonSchema = serde_json::Value;

/********************
 * Common `struct`s *
 ********************/

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommonContent {
    /// If `-` provided (default) then stdin / output from previous task is read
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<indexmap::IndexMap<String, serde_json::Value>>,
}

/******************
 * HTTP `struct`s *
 ******************/

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct HttpArgs {
    #[serde(
        deserialize_with = "de_http__uri__Uri",
        serialize_with = "ser_http___uri__Uri"
    )]
    pub url: http::uri::Uri,
    #[serde(
        deserialize_with = "de_http__method__Method",
        serialize_with = "ser_http__method__Method"
    )]
    pub method: http::method::Method,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<indexmap::IndexMap<String, serde_json_extensions::ValueNoObjOrArr>>>,
}

#[allow(non_snake_case)]
fn de_http__uri__Uri<'de, D>(deserializer: D) -> Result<http::uri::Uri, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::de::Deserialize;
    let buf = String::deserialize(deserializer)?;

    http::uri::Uri::from_str(&buf).map_err(serde::de::Error::custom)
}

#[allow(non_snake_case)]
fn ser_http___uri__Uri<S>(uri: &http::uri::Uri, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(uri.to_string().as_str())
}

#[allow(non_snake_case)]
fn de_http__method__Method<'de, D>(deserializer: D) -> Result<http::method::Method, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::de::Deserialize;
    let buf = String::deserialize(deserializer)?;

    http::method::Method::from_str(&buf).map_err(serde::de::Error::custom)
}

#[allow(non_snake_case)]
fn ser_http__method__Method<S>(method: &http::method::Method, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(method.to_string().as_str())
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct HttpCommandArgs<T = serde_json::Value> {
    pub args: HttpArgs,
    pub common_content: CommonContent,
    pub expectation: Expectation,
    #[serde(skip)]
    pub deserialize_to: T,
}

impl HttpCommandArgs {
    #[cfg(test)]
    pub(crate) fn new(
        args: HttpArgs,
        common_content: CommonContent,
        expectation: Expectation,
    ) -> Self {
        Self {
            args,
            common_content,
            expectation,
            deserialize_to: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Expectation {
    pub status_code: u16,
    pub exit_code: i32,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectationHttpClient {
    pub status_code: u16,
    pub exit_code: i32,
}

impl Default for Expectation {
    fn default() -> Self {
        Self {
            status_code: 200,
            exit_code: 0,
        }
    }
}

impl Expectation {
    pub fn is_success(&self) -> bool {
        300 > self.status_code && self.status_code >= 200
    }
}
