use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;

use crate::commands::CommandArgs;

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pipeline {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub engine_version: String,

    /// Optional environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<indexmap::IndexMap<String, either::Either<String, Vec<u8>>>>,

    /// List of pipeline stages
    pub pipe: Vec<Stage>,

    /// Map of task names to their definitions ; expected to be used in jsonref context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<indexmap::IndexMap<String, Task>>,

    /// Map of schema names to their definitions ; expected to be used in jsonref context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<HashMap<String, JsonSchema>>,
}

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub name: String,
    /// List of dependencies for this stage
    pub deps: Vec<Task>,
    /// Whether `Task`s in this stage should run sequentially
    pub sequential: bool,
}

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub commands: Vec<CommandArgs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<JsonSchema>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<JsonSchema>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<indexmap::IndexMap<String, either::Either<String, Vec<u8>>>>,
}

impl Task {
    pub fn from_task_merge_env(
        task: &Task,
        env: &indexmap::IndexMap<String, either::Either<String, Vec<u8>>>,
    ) -> Task {
        Self {
            commands: task.commands.clone(),
            input_schema: task.input_schema.clone(),
            output_schema: task.output_schema.clone(),
            env: match &task.env {
                Some(ref e) => {
                    let mut new_env = e.clone();
                    new_env.extend(env.clone());
                    Some(new_env)
                }
                None => Some(env.clone()),
            },
        }
    }
}

#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(deny_unknown_fields)]
pub struct JsonSchema {
    #[serde(flatten)]
    pub schema: serde_json::Value,
}

#[macro_export]
macro_rules! tri {
    ($e:expr $(,)?) => {
        match $e {
            core::result::Result::Ok(val) => val,
            core::result::Result::Err(err) => return core::result::Result::Err(err),
        }
    };
}

/********************
 * Common `struct`s *
 ********************/

#[derive(Clone, Debug, Default, PartialEq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommonContent {
    /// If `-` provided (default) then stdin / output from previous task is read
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "de__utf8_else_vecu8",
        serialize_with = "se__utf8_else_vecu8"
    )]
    pub content: Option<Vec<u8>>, /* Option<impl std::io::Read> */

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "de_opt_indexmap_str_or_bytes",
        serialize_with = "se_opt_indexmap_str_or_bytes"
    )]
    pub env: Option<indexmap::IndexMap<String, either::Either<String, Vec<u8>>>>,
}

#[allow(non_snake_case)]
fn de__utf8_else_vecu8<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Utf8OrBytes;

    impl<'de> serde::de::Visitor<'de> for Utf8OrBytes {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a UTF-8 string or a sequence of bytes")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.as_bytes().to_vec())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(byte) = seq.next_element()? {
                vec.push(byte);
            }
            Ok(vec)
        }
    }

    Ok(Some(deserializer.deserialize_any(Utf8OrBytes)?))
}

#[allow(non_snake_case)]
fn se__utf8_else_vecu8<S>(opt_value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;

    match opt_value {
        Some(vec) => match std::str::from_utf8(vec) {
            Ok(utf8_str) => serializer.serialize_str(utf8_str),
            Err(_) => {
                let mut seq = serializer.serialize_seq(Some(vec.len()))?;
                for &byte in vec {
                    seq.serialize_element(&byte)?;
                }
                seq.end()
            }
        },
        _ => serializer.serialize_none(),
    }
}

pub fn de_opt_indexmap_str_or_bytes<'de, D>(
    deserializer: D,
) -> Result<Option<indexmap::IndexMap<String, either::Either<String, Vec<u8>>>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct MapVisitor;

    impl<'de> serde::de::Visitor<'de> for MapVisitor {
        type Value = indexmap::IndexMap<String, either::Either<String, Vec<u8>>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map with string keys and either string or byte array values")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut index_map = indexmap::IndexMap::new();
            while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                let either_value = match value {
                    // If it's a string we treat it as Either::Left (String)
                    serde_json::Value::String(s) => either::Either::Left(s),

                    // If it's an array of numbers, we process it as Either::Right (Vec<u8>)
                    serde_json::Value::Array(arr) => {
                        let bytes: Result<Vec<u8>, _> = arr
                            .into_iter()
                            .map(|v| match v {
                                serde_json::Value::Number(n) => n
                                    .as_u64()
                                    .filter(|&val| val <= u8::MAX as u64)
                                    .map(|val| val as u8)
                                    .ok_or_else(|| serde::de::Error::custom("Invalid byte value")),
                                _ => Err(serde::de::Error::custom("Expected byte values")),
                            })
                            .collect();
                        either::Either::Right(bytes?)
                    }

                    _ => {
                        return Err(serde::de::Error::custom(
                            "Invalid value type for IndexMap value",
                        ))
                    }
                };
                index_map.insert(key, either_value);
            }
            Ok(index_map)
        }
    }

    deserializer.deserialize_option(VisitorOptionMap(MapVisitor))
}

pub struct VisitorOptionMap<T>(T);

impl<'de, T> serde::de::Visitor<'de> for VisitorOptionMap<T>
where
    T: serde::de::Visitor<'de>,
{
    type Value = Option<T::Value>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.expecting(formatter)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Some(self.0.visit_some(deserializer)?))
    }
}

pub fn se_opt_indexmap_str_or_bytes<S>(
    kv_opt: &Option<indexmap::IndexMap<String, either::Either<String, Vec<u8>>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;

    if let Some(ref index_map) = kv_opt {
        let mut map_ser = serializer.serialize_map(Some(index_map.len()))?;
        for (key, value) in index_map {
            match value {
                either::Either::Left(s) => {
                    // Serialize string directly as a string
                    map_ser.serialize_entry(key, s)?;
                }
                either::Either::Right(vec) => {
                    if let Ok(utf8_str) = std::str::from_utf8(vec) {
                        // It can be represented as a valid UTF-8 string
                        map_ser.serialize_entry(key, utf8_str)?;
                    } else {
                        // It's not valid UTF-8, serialize as a number array (i.e., sequence of bytes)
                        map_ser.serialize_entry(key, &vec)?;
                    }
                }
            }
        }
        map_ser.end()
    } else {
        serializer.serialize_none()
    }
}

/******************
 * HTTP `struct`s *
 ******************/

#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize)]
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

#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(deny_unknown_fields)]
pub struct HttpCommandArgs {
    pub args: HttpArgs,
    pub common_content: CommonContent,
    pub expectation: Expectation,
}

#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Expectation {
    pub status_code: u16,
    pub exit_code: i32,
}

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
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
