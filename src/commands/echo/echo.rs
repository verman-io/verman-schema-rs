use std::str::pattern::Pattern;

use crate::errors::VermanSchemaError;
use crate::models::CommonContent;

pub fn echo(common_content: &CommonContent) -> Result<CommonContent, VermanSchemaError> {
    let content = common_content.content.to_owned();
    let env: indexmap::IndexMap<String, serde_json::Value> = match common_content.env {
        Some(ref _env) => _env.to_owned(),
        None => indexmap::IndexMap::<String, serde_json::Value>::new(),
    };
    // There is no substitute for success
    let substitute_success =
        |input_val: &serde_json::Value| -> Result<CommonContent, VermanSchemaError> {
            let input_s = match input_val {
                serde_json::Value::String(s) => Ok(s),
                _ => Err(VermanSchemaError::NotFound("string input to provide")),
            }?;

            if input_s.is_empty() {
                return Err(VermanSchemaError::NotFound("input to provide"));
            }
            let variables = {
                let mut hm = std::collections::HashMap::<String, String>::with_capacity(env.len());
                hm.extend(env.iter().filter_map(|(k, v)| match v {
                    serde_json::Value::String(s) => Some((k.to_owned(), s.to_owned())),
                    _ => None,
                }));
                hm
            };

            let substituted = subst::substitute(input_s.as_str(), &variables)?;
            Ok(CommonContent {
                content: Some(serde_json::Value::String(String::from(substituted))),
                ..CommonContent::default()
            })
        };
    match content {
        None => match env.get("PREVIOUS_TASK_CONTENT") {
            Some(previous_task_output) => substitute_success(previous_task_output),
            None => Err(VermanSchemaError::NotFound("input to provide")),
        },
        Some(input) => match input {
            serde_json::Value::String(_) => {
                let mut input_s = input.to_owned().as_str().unwrap().to_string();
                if input_s.len() == 1 && String::from("-").is_prefix_of(input_s.as_str()) {
                    if let Some(serde_json::Value::String(previous_task_output)) =
                        env.get("PREVIOUS_TASK_CONTENT")
                    {
                        input_s = previous_task_output.to_string();
                    }
                }
                return substitute_success(&serde_json::Value::String(input_s));
            }
            serde_json::Value::Null => {
                if let Some(serde_json::Value::String(previous_task_output)) =
                    env.get("PREVIOUS_TASK_CONTENT")
                {
                    return substitute_success(&serde_json::Value::String(
                        previous_task_output.to_string(),
                    ));
                } else {
                    Ok(common_content.to_owned())
                }
            }
            _ => Ok(common_content.to_owned()),
        },
    }
}

#[cfg(test)]
#[path = "echo_test.rs"]
mod tests;
