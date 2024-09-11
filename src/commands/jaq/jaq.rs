use crate::error::VermanSchemaError;

mod jaq_utils;

pub fn jaq<'a>(
    vars: &'a mut indexmap::IndexMap<String, String>,
    filter: &str,
) -> Result<String, VermanSchemaError> {
    jaq_utils::give(serde_json::Value::Object(vars), filter)
}
