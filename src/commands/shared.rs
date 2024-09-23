use std::str::pattern::Pattern;

use crate::errors::VermanSchemaError;
use crate::models::CommonContent;

pub(crate) fn interpolate_input_with_env(
    common_content: &CommonContent,
) -> Result<CommonContent, VermanSchemaError> {
    let content = common_content.content.to_owned();
    let env: indexmap::IndexMap<String, serde_json::Value> = match common_content.env {
        Some(ref _env) => _env.to_owned(),
        None => indexmap::IndexMap::<String, serde_json::Value>::new(),
    };
    // There is no substitute for success
    let substitute_success = |input_val: &serde_json::Value| -> Result<CommonContent, VermanSchemaError> {
        let input_s = match input_val {
            serde_json::Value::String(s) => Ok(s),
            _ => Err(VermanSchemaError::NotFound("string input to provide")),
        }?;

        if input_s.is_empty() {
            return Err(VermanSchemaError::NotFound("input to provide"));
        }
        let variables = make_subst_map(&env);

        let substituted = subst::substitute(input_s.as_str(), &variables)?;
        Ok(CommonContent {
            content: Some(serde_json::Value::String(String::from(substituted))),
            env: Some(env.clone()),
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
                substitute_success(&serde_json::Value::String(input_s))
            }
            serde_json::Value::Null => {
                if let Some(serde_json::Value::String(previous_task_output)) =
                    env.get("PREVIOUS_TASK_CONTENT")
                {
                    substitute_success(&serde_json::Value::String(previous_task_output.to_string()))
                } else {
                    Ok(common_content.to_owned())
                }
            }
            _ => Ok(common_content.to_owned()),
        },
    }
}

pub(crate) fn make_subst_map(
    env: &indexmap::IndexMap<String, serde_json::Value>,
) -> std::collections::HashMap<String, String> {
    let mut hm0 = std::collections::HashMap::<String, String>::with_capacity(env.len());
    let mut hm1 = std::collections::HashMap::<String, String>::new();
    hm0.extend(env.iter().filter_map(|(k, v)| match v {
        serde_json::Value::String(s) => Some((k.to_owned(), s.to_owned())),
        serde_json::Value::Object(m) => {
            hm1.extend(
                m.iter()
                    .map(|(key, val)| (key.to_owned(), serde_json::to_string(val).unwrap())),
            );
            None
        }
        _ => None,
    }));
    hm0.extend(hm1);
    hm0
}
