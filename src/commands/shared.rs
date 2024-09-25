use std::str::pattern::Pattern;

use crate::commands::command::CommandKey;
use crate::errors::VermanSchemaError;
use crate::models::CommonContent;

pub(crate) fn interpolate_input_with_env(
    common_content: &CommonContent,
    ignore_errors: bool
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

        let substituted = match subst::substitute(input_s.as_str(), &variables) {
            Ok(s) => s,
            Err(e) => {
                if !ignore_errors {
                    return Err(e.into())
                }
                input_s.to_owned()
            }
        };
        Ok(CommonContent {
            content: Some(serde_json::Value::String(String::from(substituted))),
            env: Some(env.clone()),
        })
    };
    match content {
        None => match env.get(CommandKey::PreviousContent.to_string().as_str()) {
            Some(previous_task_output) => substitute_success(previous_task_output),
            None => Err(VermanSchemaError::NotFound("input to provide")),
        },
        Some(input) => match input {
            serde_json::Value::String(s) => {
                let mut input_s = s;
                if input_s.len() == 1 && String::from("-").is_prefix_of(input_s.as_str()) {
                    if let Some(serde_json::Value::String(previous_task_output)) =
                        env.get(CommandKey::PreviousContent.to_string().as_str())
                    {
                        input_s = previous_task_output.to_string();
                    }
                }
                substitute_success(&serde_json::Value::String(input_s))
            }
            serde_json::Value::Null => {
                if let Some(serde_json::Value::String(previous_task_output)) =
                    env.get(CommandKey::PreviousContent.to_string().as_str())
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

pub(crate) fn interpolate_input_else_get_prior_output(
    common_content: &CommonContent,
    ignore_substitution_errors: bool
) -> Result<CommonContent, VermanSchemaError> {
    let common_content_out = {
        let mut common = match interpolate_input_with_env(common_content, ignore_substitution_errors) {
            Ok(out) => out,
            Err(e) => match e {
                VermanSchemaError::NotFound(_) => CommonContent {
                    env: common_content.env.clone(),
                    content: None,
                },
                err @ _ => return Err(err),
            },
        };
        if common.content.is_none() {
            if let Some(ref env) = common.env {
                common.content = env
                    .get(CommandKey::PreviousContent.to_string().as_str())
                    .map(|v| v.to_owned())
            }
        }
        common
    };
    Ok(common_content_out)
}

pub(crate) fn make_subst_map(
    env: &indexmap::IndexMap<String, serde_json::Value>,
) -> std::collections::HashMap<String, String> {
    let mut hm = std::collections::HashMap::<String, String>::with_capacity(env.len());
    hm.extend(env.iter().filter_map(|(k, v)| match v {
            serde_json::Value::String(s) => Some((k.to_owned(), s.to_owned())),
            serde_json::Value::Number(n) => Some((k.to_owned(), n.to_string())),
            v @ _ => Some((k.to_owned(), serde_json::to_string(v).unwrap())),
    }));
    hm
}

pub fn merge_env(
    inferior: &mut indexmap::IndexMap<String, serde_json::Value>,
    superior: &Option<indexmap::IndexMap<String, serde_json::Value>>,
) {
    if let Some(env) = superior {
        inferior.extend(env.to_owned())
    }
}
